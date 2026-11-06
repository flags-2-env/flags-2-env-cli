"use strict";
// Finite decision-policy model matching src/catalog.rs default_source_order().
// Concrete TOML/CLI parsing and .env file traversal require separate refinement.
const NORMAL=["flags","env_shell","env_file"];
const DOTENV_OVERRIDE=["flags","env_file","env_shell"];
const SOURCES=["flags","env_shell","env_file"];
function must(v,m){if(!v)throw Error("flags2env model: "+m);}
function order(override){return override?DOTENV_OVERRIDE:NORMAL;}
function resolve(present,enableDotenv,override){
 const enabled={...present,env_file:enableDotenv&&present.env_file};
 for(const key of order(override))if(enabled[key])return key;
 return "unset";
}
function run(){
 let examined=0,wins={flags:0,env_shell:0,env_file:0,unset:0};
 let overrideDifference=0;
 for(let mask=0;mask<8;mask++){
  for(const enableDotenv of [false,true]){
   const present={
    flags:!!(mask&1),
    env_shell:!!(mask&2),
    env_file:!!(mask&4)
   };
   const def=resolve(present,enableDotenv,false);
   const override=resolve(present,enableDotenv,true);
   for(const winner of [def,override]){
    examined++;wins[winner]++;
    if(winner!=="unset")must(present[winner]&&(winner!=="env_file"||enableDotenv),"source invented by precedence");
   }
   if(present.flags)must(def==="flags"&&override==="flags","flags lost highest priority");
   if(!enableDotenv)must(def!=="env_file"&&override!=="env_file","dotenv read despite opt-out");
   if(def!==override)overrideDifference++;
  }
 }
 for(const [name,count]of Object.entries(wins))must(count>0,"unreached source "+name);
 must(overrideDifference>0,"dotenv override path never exercised");
 const forged={flags:false,env_shell:false,env_file:false};
 must(resolve(forged,false,false)==="unset","invented missing config");
 // Negative mutation would force dotenv when disabled: explicitly reject as a violation.
 const mutatedWinner="env_file";
 must(!(!false&&mutatedWinner!=="env_file"),"invalid fixture");
 must(mutatedWinner!==resolve({flags:false,env_shell:false,env_file:true},false,false),
  "negative mutation did not detect disabled dotenv");
 return {combinations:16,decisions:examined,winners:wins,overrideDifference,claim:"finite precedence policy only"};
}
const result=run();
if(typeof console!=="undefined")console.log(JSON.stringify(result));

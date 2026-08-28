// src/env/env.dart — service overlay. Edit defaults here; regenerate generated.dart from flags-2-env.

import 'generated.dart' as generated;

const Map<String, String> defaults = {
};

/// Service defaults, then flags-2-env overlay (`.env` vs process env vs argv).
Map<String, String> load() {
  return {
    ...defaults,
    ...generated.loadEnvMapFromOs(),
  };
}

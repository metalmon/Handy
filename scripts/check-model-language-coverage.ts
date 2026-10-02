import { readFileSync } from "node:fs";
import {
  MODEL_CAPABILITY_LANGUAGES,
  supportsLanguageCode,
} from "../src/lib/constants/languages.ts";

const MODEL_MANAGER = "src-tauri/src/managers/model.rs";

const failures: string[] = [];

// These model-code variants intentionally share one persisted UI intent. Keep
// this assertion here so a language rename cannot silently break continuity for
// users who already have `no` stored.
if (
  !supportsLanguageCode(["nb"], "no") ||
  !supportsLanguageCode(["no"], "nb")
) {
  failures.push(
    "Norwegian intent `no` must remain equivalent to model code `nb`",
  );
}

// The model catalog is gone: this build ships exactly one model, GigaAM v3,
// declared inline in the model manager. Read its supported_languages straight
// from the registry so the check cannot drift away from the real capability.
const manager = readFileSync(MODEL_MANAGER, "utf8");
const bundled = manager.match(
  /id:\s*"gigaam-v3-e2e-ctc"[\s\S]*?supported_languages:\s*vec!\[([^\]]*)\]/,
);

if (!bundled) {
  failures.push(
    "could not find supported_languages for the bundled gigaam-v3-e2e-ctc entry in " +
      MODEL_MANAGER,
  );
} else {
  const modelLanguages = [...bundled[1].matchAll(/"([^"]+)"/g)].map(
    (match) => match[1],
  );

  if (modelLanguages.length === 0) {
    failures.push("the bundled model declares no supported languages");
  }

  for (const modelLanguage of modelLanguages) {
    const matchingIntents = MODEL_CAPABILITY_LANGUAGES.filter((language) =>
      supportsLanguageCode([modelLanguage], language.value),
    );

    if (matchingIntents.length !== 1) {
      const matchSummary =
        matchingIntents.length === 0
          ? "no frontend language intent"
          : `ambiguous intents: ${matchingIntents
              .map((language) => language.value)
              .join(", ")}`;
      failures.push(`gigaam-v3-e2e-ctc: ${modelLanguage} (${matchSummary})`);
    }
  }

  if (modelLanguages.length === 1) {
    console.log(
      `Model language coverage: gigaam-v3-e2e-ctc declares ${modelLanguages.join(", ")}, ` +
        `mapped to ${MODEL_CAPABILITY_LANGUAGES.length} frontend intents`,
    );
  }
}

if (failures.length > 0) {
  console.error("Model language coverage check failed:\n");
  for (const failure of failures) console.error(`  - ${failure}`);
  process.exit(1);
}

console.log(
  "Model language coverage: all bundled model codes map to exactly one frontend intent",
);

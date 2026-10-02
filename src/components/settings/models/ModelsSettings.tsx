import React from "react";
import { useTranslation } from "react-i18next";
import { Check, HardDrive } from "lucide-react";
import { useModelStore } from "@/stores/modelStore";
import {
  getTranslatedModelDescription,
  getTranslatedModelName,
} from "@/lib/utils/modelTranslation";
import { formatModelSize } from "@/lib/utils/format";
import Badge from "../../ui/Badge";

export const ModelsSettings: React.FC = () => {
  const { t } = useTranslation();
  const models = useModelStore((state) => state.models);
  const currentModel = useModelStore((state) => state.currentModel);
  const loading = useModelStore((state) => state.loading);

  // This build ships exactly one model, so there is nothing to switch or pick:
  // show whichever entry the backend reports as active.
  const activeModel =
    models.find((model) => model.id === currentModel) ?? models[0];

  if (loading) {
    return (
      <div className="max-w-3xl w-full mx-auto">
        <div className="flex items-center justify-center py-16">
          <div className="w-8 h-8 border-2 border-logo-primary border-t-transparent rounded-full animate-spin" />
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-3xl w-full mx-auto space-y-4">
      <div className="mb-4">
        <h1 className="text-xl font-semibold mb-2">
          {t("settings.models.title")}
        </h1>
        <p className="text-sm text-text/60">
          {t("settings.models.description")}
        </p>
      </div>

      <div className="space-y-3">
        <h2 className="text-sm font-medium text-text/60">
          {t("settings.models.yourModels")}
        </h2>

        {activeModel ? (
          <div className="flex flex-col rounded-xl px-4 py-3 gap-2 border-2 border-logo-primary/50 bg-logo-primary/10">
            <div className="flex items-center gap-3 flex-wrap">
              <h3 className="text-base font-semibold text-text">
                {getTranslatedModelName(activeModel, t)}
              </h3>
              <Badge variant="primary">
                <Check className="w-3 h-3 mr-1" />
                {t("modelSelector.active")}
              </Badge>
            </div>
            <p className="text-text/60 text-sm leading-relaxed">
              {getTranslatedModelDescription(activeModel, t)}
            </p>
            <hr className="w-full border-mid-gray/20" />
            <div className="flex items-center gap-1.5 text-xs text-text/50">
              <HardDrive className="w-3.5 h-3.5" />
              <span>{formatModelSize(Number(activeModel.size_mb))}</span>
            </div>
          </div>
        ) : (
          <div className="text-center py-8 text-text/50">
            {t("settings.models.noModelsMatch")}
          </div>
        )}
      </div>
    </div>
  );
};
import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { listen } from "@tauri-apps/api/event";
import { commands } from "@/bindings";
import { getTranslatedModelName } from "../../lib/utils/modelTranslation";
import { useModelStore } from "../../stores/modelStore";
import ModelStatusIndicator from "./ModelStatusIndicator";

import { ModelStateEvent } from "@/lib/types/events";

type ModelStatus = "ready" | "loading" | "error" | "unloaded" | "none";

interface ModelSelectorProps {
  onError?: (error: string) => void;
}

const ModelSelector: React.FC<ModelSelectorProps> = ({ onError }) => {
  const { t } = useTranslation();
  const models = useModelStore((state) => state.models);
  const currentModel = useModelStore((state) => state.currentModel);

  const [modelStatus, setModelStatus] = useState<ModelStatus>("unloaded");
  const [modelError, setModelError] = useState<string | null>(null);

  // Check model status when currentModel changes
  useEffect(() => {
    const checkStatus = async () => {
      if (currentModel) {
        try {
          const statusResult = await commands.getTranscriptionModelStatus();
          if (statusResult.status === "ok") {
            setModelStatus(
              statusResult.data === currentModel ? "ready" : "unloaded",
            );
          }
        } catch {
          setModelStatus("error");
          setModelError("Failed to check model status");
          onError?.("Failed to check model status");
        }
      } else {
        setModelStatus("none");
      }
    };
    checkStatus();
  }, [currentModel, onError]);

  useEffect(() => {
    // Listen for model loading lifecycle events
    const unlisten = listen<ModelStateEvent>("model-state-changed", (event) => {
      const { event_type, error } = event.payload;
      switch (event_type) {
        case "loading_started":
          setModelStatus("loading");
          setModelError(null);
          break;
        case "loading_completed":
          setModelStatus("ready");
          setModelError(null);
          break;
        case "loading_failed":
          setModelStatus("error");
          setModelError(error || "Failed to load model");
          break;
        case "unloaded":
          setModelStatus("unloaded");
          setModelError(null);
          break;
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const getModelDisplayText = (): string => {
    const currentModelInfo = models.find((m) => m.id === currentModel);
    const modelName = currentModelInfo
      ? getTranslatedModelName(currentModelInfo, t)
      : null;

    switch (modelStatus) {
      case "ready":
        return modelName ?? t("modelSelector.modelReady");
      case "loading":
        return modelName
          ? t("modelSelector.loading", { modelName })
          : t("modelSelector.loadingGeneric");
      case "error":
        return modelError || t("modelSelector.modelError");
      case "unloaded":
        return modelName ?? t("modelSelector.modelUnloaded");
      case "none":
        return t("modelSelector.noModelDownloadRequired");
      default:
        return modelName ?? t("modelSelector.modelUnloaded");
    }
  };

  return (
    <ModelStatusIndicator
      status={modelStatus}
      displayText={getModelDisplayText()}
    />
  );
};

export default ModelSelector;

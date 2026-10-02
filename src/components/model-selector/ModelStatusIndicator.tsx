import React from "react";
import { useTranslation } from "react-i18next";

type ModelStatus = "ready" | "loading" | "error" | "unloaded" | "none";

interface ModelStatusIndicatorProps {
  status: ModelStatus;
  displayText: string;
  className?: string;
}

const ModelStatusIndicator: React.FC<ModelStatusIndicatorProps> = ({
  status,
  displayText,
  className = "",
}) => {
  const { t } = useTranslation();

  const getStatusColor = (modelStatus: ModelStatus): string => {
    switch (modelStatus) {
      case "ready":
        return "bg-green-400";
      case "loading":
        return "bg-yellow-400 animate-pulse";
      case "error":
        return "bg-red-400";
      case "unloaded":
        return "bg-mid-gray/60";
      case "none":
        return "bg-red-400";
      default:
        return "bg-mid-gray/60";
    }
  };

  return (
    <div className={`flex items-center gap-2 ${className}`}>
      <div className={`w-2 h-2 rounded-full ${getStatusColor(status)}`} />
      <span
        className="max-w-28 truncate"
        title={t("modelSelector.statusTitle", { modelStatus: displayText })}
      >
        {displayText}
      </span>
    </div>
  );
};

export default ModelStatusIndicator;

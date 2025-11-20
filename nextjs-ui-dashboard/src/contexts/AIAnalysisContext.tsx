import React, { createContext, useContext, ReactNode } from "react";
import { useAIAnalysis, AIAnalysisHook } from "@/hooks/useAIAnalysis";

const AIAnalysisContext = createContext<AIAnalysisHook | undefined>(undefined);

export const AIAnalysisProvider: React.FC<{ children: ReactNode }> = ({
  children,
}) => {
  const aiAnalysis = useAIAnalysis();

  return (
    <AIAnalysisContext.Provider value={aiAnalysis}>
      {children}
    </AIAnalysisContext.Provider>
  );
};

export const useAIAnalysisContext = (): AIAnalysisHook => {
  const context = useContext(AIAnalysisContext);
  if (!context) {
    throw new Error(
      "useAIAnalysisContext must be used within AIAnalysisProvider"
    );
  }
  return context;
};

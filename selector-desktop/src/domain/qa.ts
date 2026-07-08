export interface QaCoverageAudit {
  status: "pass" | "fail";
  totalChapters: number;
  doneChapters: number;
  missingChapters: string[];
  items: QaCoverageItem[];
  checks: QaCheck[];
}

export interface QaCoverageItem {
  id: string;
  chapter: string;
  status: string;
  sourcePage: string;
  implementationShape: string;
}

export interface QaCheck {
  label: string;
  passed: boolean;
  detail: string;
}

export interface QaRegressionReport {
  status: "pass" | "fail";
  totalCases: number;
  passedCases: number;
  failedCases: number;
  groups: QaRegressionGroup[];
}

export interface QaRegressionGroup {
  label: string;
  totalCases: number;
  passedCases: number;
  failedCases: number;
  cases: QaRegressionCaseResult[];
}

export interface QaRegressionCaseResult {
  name: string;
  moduleId: string;
  status: "pass" | "fail";
  detail: string;
}

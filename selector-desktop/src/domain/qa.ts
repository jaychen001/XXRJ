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

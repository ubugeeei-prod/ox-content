import * as vscode from "vscode";

export type PreviewPayload = {
  html: string;
  title: string;
};

export type PreviewEntry = {
  documentUri: string;
  panel: vscode.WebviewPanel;
};

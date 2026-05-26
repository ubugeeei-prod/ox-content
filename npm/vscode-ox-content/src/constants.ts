export const COMMAND_INSERT_TABLE = "oxContent.insertTable";
export const COMMAND_INSERT_CODE_FENCE = "oxContent.insertCodeFence";
export const COMMAND_INSERT_CALLOUT = "oxContent.insertCallout";
export const COMMAND_OPEN_PREVIEW = "oxContent.openPreview";
export const SERVER_COMMAND_PREVIEW_HTML = "oxContent.previewHtml";
export const SERVER_COMMAND_PREVIEW_SUBSCRIBE = "oxContent.previewSubscribe";
export const SERVER_COMMAND_PREVIEW_UNSUBSCRIBE = "oxContent.previewUnsubscribe";

/** Notification pushed by the LSP whenever a subscribed document
 * changes. Replaces the polling refresh path. */
export const NOTIFICATION_PREVIEW_DID_CHANGE = "oxContent/previewDidChange";

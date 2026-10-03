export async function prompts() {
  const ui = await import("@clack/prompts");
  const checked = (value) => {
    if (ui.isCancel(value)) {
      ui.cancel("Cancelled. No further changes made.");
      const error = new Error("Setup cancelled");
      error.code = "OXCT_CANCELLED";
      throw error;
    }
    return value;
  };
  return { ...ui, checked };
}

export function optionValue(args, index, flag) {
  const value = args[index];
  if (!value || value.startsWith("-")) throw new Error(`${flag} requires a value`);
  return value;
}

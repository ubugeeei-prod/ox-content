declare module "virtual:ox-content-solid/html-host/modules" {
  export interface SolidHtmlHostClientModule {
    name: string;
    moduleId: string;
    exportName: string;
  }

  export type SolidHtmlHostClientModules = Readonly<Record<string, () => Promise<unknown>>>;

  export const modules: SolidHtmlHostClientModules;
  export const clientModules: readonly SolidHtmlHostClientModule[];
  export default modules;
}

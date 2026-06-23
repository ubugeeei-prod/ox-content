declare module "virtual:ox-content/collections" {
  type Entry = import("./types").CollectionEntry;
  type Builder<T extends Entry = Entry> = import("./types").CollectionQueryBuilder<T>;
  type Operator = import("./types").CollectionQueryOperator;

  export const collections: Record<string, Entry[]>;
  export const collectionNames: string[];
  export class CollectionQueryBuilderImpl<T extends Entry = Entry> {
    constructor(rows: T[]);
    path(path: string): Builder<T>;
    select<K extends keyof T>(...fields: K[]): Builder<Pick<T, K> & Entry>;
    where(field: keyof T | string, operator: Operator, value?: unknown): this;
    where(field: keyof T | string, value: unknown): this;
    andWhere(factory: (query: Builder<T>) => void): this;
    orWhere(factory: (query: Builder<T>) => void): this;
    order(field: keyof T | string, direction?: "ASC" | "DESC"): this;
    limit(limit: number): this;
    skip(skip: number): this;
    all(): Promise<T[]>;
    first(): Promise<T | null>;
    count(): Promise<number>;
  }
  export { CollectionQueryBuilderImpl as CollectionQueryBuilder };
  export function getCollection(name: string): Entry[];
  export function queryCollection<T extends Entry = Entry>(name: string): Builder<T>;
  const api: {
    collections: Record<string, Entry[]>;
    collectionNames: string[];
    getCollection: typeof getCollection;
    queryCollection: typeof queryCollection;
  };
  export default api;
}

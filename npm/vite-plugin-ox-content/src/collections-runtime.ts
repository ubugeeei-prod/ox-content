import type { CollectionManifest } from "./types";

const runtime = String.raw`
function createGetter(field) {
  const direct = String(field);
  const nested = direct.includes(".") ? direct.split(".") : undefined;
  if (!nested) return (row) => row[direct];
  return (row) => {
    if (direct in row) return row[direct];
    let value = row;
    for (const key of nested) {
      if (value == null) return undefined;
      value = value[key];
    }
    return value;
  };
}

function normalizePath(value) {
  const path = String(value || "/");
  if (path === "/") return path;
  return path.startsWith("/") ? path.replace(/\/+$/, "") : "/" + path.replace(/\/+$/, "");
}

function likePattern(value) {
  const escaped = String(value).replace(/[\\^$.*+?()[\]{}|]/g, "\\$&");
  return new RegExp("^" + escaped.replace(/%/g, ".*").replace(/_/g, ".") + "$", "i");
}

const pathIndexes = new WeakMap();

function getPathIndex(rows) {
  let index = pathIndexes.get(rows);
  if (index) return index;
  index = new Map();
  for (const row of rows) {
    if (!index.has(row.path)) index.set(row.path, row);
  }
  pathIndexes.set(rows, index);
  return index;
}

const collator = new Intl.Collator(undefined, { numeric: true, sensitivity: "base" });

function compare(left, right) {
  if (left == null && right == null) return 0;
  if (left == null) return -1;
  if (right == null) return 1;
  if (typeof left === "number" && typeof right === "number") return left - right;
  if (left instanceof Date || right instanceof Date) {
    return new Date(left).getTime() - new Date(right).getTime();
  }
  return collator.compare(String(left), String(right));
}

function createPredicate(field, operator, value) {
  let op = String(operator ?? "=").toUpperCase();
  let expected = value;
  if (arguments.length === 2) {
    op = "=";
    expected = operator;
  }

  const get = createGetter(field);
  const lookup = (op === "IN" || op === "NOT IN") && Array.isArray(expected)
    ? new Set(expected)
    : undefined;
  const pattern = (op === "LIKE" || op === "NOT LIKE") ? likePattern(expected) : undefined;

  return (row) => {
    const actual = get(row);
    switch (op) {
      case "=":
      case "==":
        return actual === expected;
      case "!=":
      case "<>":
        return actual !== expected;
      case ">":
        return compare(actual, expected) > 0;
      case ">=":
        return compare(actual, expected) >= 0;
      case "<":
        return compare(actual, expected) < 0;
      case "<=":
        return compare(actual, expected) <= 0;
      case "IN":
        return lookup ? lookup.has(actual) : false;
      case "NOT IN":
        return lookup ? !lookup.has(actual) : false;
      case "BETWEEN":
        return Array.isArray(expected) && expected.length >= 2
          ? compare(actual, expected[0]) >= 0 && compare(actual, expected[1]) <= 0
          : false;
      case "NOT BETWEEN":
        return Array.isArray(expected) && expected.length >= 2
          ? compare(actual, expected[0]) < 0 || compare(actual, expected[1]) > 0
          : false;
      case "IS NULL":
        return actual == null;
      case "IS NOT NULL":
        return actual != null;
      case "LIKE":
        return pattern.test(String(actual ?? ""));
      case "NOT LIKE":
        return !pattern.test(String(actual ?? ""));
      default:
        throw new Error("Unsupported collection query operator: " + op);
    }
  };
}

class QueryGroup {
  constructor(rows) {
    this.rows = rows;
    this.conditions = [];
  }

  where(field, operator, value) {
    const test =
      arguments.length === 2
        ? createPredicate(field, operator)
        : createPredicate(field, operator, value);
    this.conditions.push({ join: "and", test });
    return this;
  }

  andWhere(factory) {
    const group = new QueryGroup(this.rows);
    factory(group);
    this.conditions.push({ join: "and", test: (row) => group.test(row) });
    return this;
  }

  orWhere(factory) {
    const group = new QueryGroup(this.rows);
    factory(group);
    this.conditions.push({ join: "or", test: (row) => group.test(row) });
    return this;
  }

  test(row) {
    let matched = true;
    for (const condition of this.conditions) {
      matched =
        condition.join === "or" ? matched || condition.test(row) : matched && condition.test(row);
    }
    return matched;
  }
}

class CollectionQueryBuilder extends QueryGroup {
  constructor(rows) {
    super(rows);
    this.orders = [];
    this.selected = undefined;
    this.offset = 0;
    this.max = undefined;
    this.exactPath = undefined;
  }

  path(path) {
    this.exactPath = normalizePath(path);
    return this.where("path", "=", this.exactPath);
  }

  select(...fields) {
    this.selected = fields.map((field) => ({ field, get: createGetter(field) }));
    return this;
  }

  order(field, direction = "ASC") {
    this.orders.push({ get: createGetter(field), direction: String(direction).toUpperCase() });
    return this;
  }

  limit(limit) {
    this.max = Math.max(0, Number(limit) || 0);
    return this;
  }

  skip(skip) {
    this.offset = Math.max(0, Number(skip) || 0);
    return this;
  }

  materialize() {
    let rows;
    if (!this.conditions.length) {
      rows = this.rows;
    } else if (!this.orders.length && this.max !== undefined) {
      rows = [];
      const end = this.offset + this.max;
      for (const row of this.rows) {
        if (this.test(row)) {
          rows.push(row);
          if (rows.length >= end) break;
        }
      }
    } else {
      rows = this.rows.filter((row) => this.test(row));
    }
    if (this.orders.length) {
      rows = [...rows].sort((left, right) => {
        for (const order of this.orders) {
          const result = compare(order.get(left), order.get(right));
          if (result !== 0) return order.direction === "DESC" ? -result : result;
        }
        return 0;
      });
    }
    if (this.offset || this.max !== undefined) {
      rows = rows.slice(this.offset, this.max === undefined ? undefined : this.offset + this.max);
    }
    if (!this.selected) return rows;
    return rows.map((row) => this.project(row));
  }

  project(row) {
    if (!this.selected) return row;
    const selected = {};
    for (const field of this.selected) selected[field.field] = field.get(row);
    return selected;
  }

  async all() {
    return this.materialize();
  }

  async first() {
    if (this.max === 0) return null;
    if (
      this.exactPath !== undefined &&
      this.conditions.length === 1 &&
      !this.orders.length &&
      this.offset === 0
    ) {
      const row = getPathIndex(this.rows).get(this.exactPath);
      return row && this.test(row) ? this.project(row) : null;
    }
    if (this.orders.length) return this.materialize()[0] ?? null;
    let skipped = 0;
    for (const row of this.rows) {
      if (this.conditions.length && !this.test(row)) continue;
      if (skipped++ < this.offset) continue;
      return this.project(row);
    }
    return null;
  }

  async count() {
    if (!this.conditions.length) return this.rows.length;
    let count = 0;
    for (const row of this.rows) {
      if (this.test(row)) count++;
    }
    return count;
  }
}

export function getCollection(name) {
  return collections[name] ? [...collections[name]] : [];
}

export function queryCollection(name) {
  return new CollectionQueryBuilder(collections[name] || []);
}

export const collectionNames = Object.keys(collections);
export { CollectionQueryBuilder };
export default { collections, collectionNames, getCollection, queryCollection };
`;

export function generateCollectionsModule(manifest: CollectionManifest): string {
  return `const collections = ${JSON.stringify(manifest.collections)};\n${runtime}`;
}

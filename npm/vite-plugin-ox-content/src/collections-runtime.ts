import type { CollectionManifest } from "./types";

const runtime = String.raw`
function getValue(row, field) {
  if (field in row) return row[field];
  return String(field)
    .split(".")
    .reduce((value, key) => (value == null ? undefined : value[key]), row);
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

function compare(left, right) {
  if (left == null && right == null) return 0;
  if (left == null) return -1;
  if (right == null) return 1;
  if (typeof left === "number" && typeof right === "number") return left - right;
  if (left instanceof Date || right instanceof Date) {
    return new Date(left).getTime() - new Date(right).getTime();
  }
  return String(left).localeCompare(String(right), undefined, {
    numeric: true,
    sensitivity: "base",
  });
}

function createPredicate(field, operator, value) {
  let op = String(operator ?? "=").toUpperCase();
  let expected = value;
  if (arguments.length === 2) {
    op = "=";
    expected = operator;
  }

  return (row) => {
    const actual = getValue(row, field);
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
        return Array.isArray(expected) && expected.includes(actual);
      case "NOT IN":
        return Array.isArray(expected) && !expected.includes(actual);
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
        return likePattern(expected).test(String(actual ?? ""));
      case "NOT LIKE":
        return !likePattern(expected).test(String(actual ?? ""));
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
  }

  path(path) {
    return this.where("path", "=", normalizePath(path));
  }

  select(...fields) {
    this.selected = fields;
    return this;
  }

  order(field, direction = "ASC") {
    this.orders.push({ field, direction: String(direction).toUpperCase() });
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
    let rows = this.conditions.length ? this.rows.filter((row) => this.test(row)) : this.rows;
    if (this.orders.length) {
      rows = [...rows].sort((left, right) => {
        for (const order of this.orders) {
          const result = compare(getValue(left, order.field), getValue(right, order.field));
          if (result !== 0) return order.direction === "DESC" ? -result : result;
        }
        return 0;
      });
    }
    if (this.offset || this.max !== undefined) {
      rows = rows.slice(this.offset, this.max === undefined ? undefined : this.offset + this.max);
    }
    if (!this.selected) return rows;
    return rows.map((row) => {
      const selected = {};
      for (const field of this.selected) selected[field] = getValue(row, field);
      return selected;
    });
  }

  async all() {
    return this.materialize();
  }

  async first() {
    return this.materialize()[0] ?? null;
  }

  async count() {
    return this.conditions.length
      ? this.rows.filter((row) => this.test(row)).length
      : this.rows.length;
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

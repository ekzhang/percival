/** An entry in a relation. */
export type Row = Record<string, unknown> & { __type__: "row" };

/** A relation. */
export type Relation = Array<Row>;

/** Typecast an array of objects to a relation. */
export function Relation(objs: object[]): Relation {
  return objs as Relation;
}

/** A set of named relations, as eg the output of a cell. */
export type RelationSet = Record<string, Relation>;

const RenderedElementKey = "__percivalRenderedElement__";

export type RenderedElement = {
  [RenderedElementKey]: true;
  outerHTML: string;
};

export function RenderedElement(elementLike: {
  outerHTML: string;
}): RenderedElement {
  return {
    [RenderedElementKey]: true,
    outerHTML: elementLike.outerHTML,
  };
}

export function isRenderedElement(value: unknown): value is RenderedElement {
  return Boolean(
    value instanceof Element ||
      (typeof value === "object" && value && RenderedElementKey in value),
  );
}

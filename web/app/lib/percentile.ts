export interface PercentileStat {
  percentiles: [number, number][];
}

/**
 * Finds the percentile of a given value in a percentile distribution.
 *
 * @param stat - The PercentileStat object with sorted percentiles.
 * @param value - The value to find the percentile for.
 * @param reverse - If true, interprets lower values as "better".
 * @returns The percentile (between 0 and 1).
 */
export function findPercentile(
  stat: PercentileStat,
  value: number,
  reverse = false
): number {
  // thanks gpt
  const points = stat.percentiles;

  if (points.length === 0) {
    throw new Error("No percentile data available.");
  }

  const getValue = (p: [number, number]) => p[1];
  const getPercentile = (p: [number, number]) => p[0];

  // If reverse is true, we flip the value comparisons.
  const compare = reverse
    ? (a: number, b: number) => a >= b
    : (a: number, b: number) => a <= b;

  // Handle edge cases
  const first = points[0];
  const last = points[points.length - 1];

  if (compare(value, getValue(first))) {
    return getPercentile(first);
  }

  if (!compare(value, getValue(last))) {
    return getPercentile(last);
  }

  // Search for two bounding points
  for (let i = 0; i < points.length - 1; i++) {
    const [p1, v1] = points[i];
    const [p2, v2] = points[i + 1];

    const lower = reverse ? v2 : v1;
    const upper = reverse ? v1 : v2;

    if (value >= lower && value <= upper) {
      const t = (value - lower) / (upper - lower);
      return reverse ? p2 + (p1 - p2) * t : p1 + (p2 - p1) * t;
    }
  }

  // Should not reach here if data is sorted and handled correctly
  throw new Error(
    "Value is out of range or percentiles are not sorted properly."
  );
}

/**
 * Finds the approximate value for a given percentile.
 *
 * @param stat - The PercentileStat object with sorted percentiles.
 * @param targetPercentile - The percentile to estimate (between 0 and 1).
 * @param reverse - If true, treats lower values as "better".
 * @returns The interpolated value.
 */
export function findValueAtPercentile(
  stat: PercentileStat,
  targetPercentile: number,
  reverse = false
): number {
  const points = stat.percentiles;

  if (points.length === 0) {
    throw new Error("No percentile data available.");
  }

  if (targetPercentile <= points[0][0]) {
    return points[0][1];
  }

  if (targetPercentile >= points[points.length - 1][0]) {
    return points[points.length - 1][1];
  }

  for (let i = 0; i < points.length - 1; i++) {
    const [p1, v1] = points[i];
    const [p2, v2] = points[i + 1];

    if (targetPercentile >= p1 && targetPercentile <= p2) {
      const t = (targetPercentile - p1) / (p2 - p1);
      const value = v1 + (v2 - v1) * t;
      return reverse ? v2 - (v2 - v1) * t : value;
    }
  }

  throw new Error("Percentile out of range or data not sorted correctly.");
}

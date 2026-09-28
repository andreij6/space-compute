const NO_BURN = 4_294_967_295;

export const runway = (n: number, unit: string): string => (!Number.isFinite(n) || n >= NO_BURN ? 'no burn yet' : `${n} ${unit}`);

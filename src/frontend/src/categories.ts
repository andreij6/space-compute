export const DISCOVERY_CATEGORIES: { id: string; label: string }[] = [
  { id: 'lens', label: 'Gravitational Lens' },
  { id: 'merger', label: 'Galaxy Merger' },
  { id: 'ring', label: 'Ring Galaxy' },
  { id: 'red_dot', label: 'Little Red Dot' },
  { id: 'clumpy', label: 'Clumpy Disk' },
  { id: 'tidal', label: 'Tidal Feature' },
  { id: 'artifact', label: 'Sensor Artifact' },
];

export const categoryLabel = (id: string): string =>
  DISCOVERY_CATEGORIES.find((c) => c.id === id)?.label ?? id;

export const formatNs = (ns: bigint): string => new Date(Number(ns / 1_000_000n)).toLocaleDateString();

const svgs = import.meta.glob<string>('../../assets/*/*.svg', { eager: true, query: '?url', import: 'default' });

export const svgAsset = (group: string, name: string): string | undefined => svgs[`../../assets/${group}/${name}.svg`];

export const tierAsset = (tier: number): string | undefined => {
  const prefix = `../../assets/tiers/tier_${tier}_`;
  return Object.entries(svgs).find(([path]) => path.startsWith(prefix))?.[1];
};

import { categoryLabel } from '../../categories';
import { svgAsset } from './assets';

const FILE_BY_CATEGORY: Record<string, string> = {
  lens: 'lensed_arc',
  merger: 'merger_interaction',
  ring: 'ring',
  red_dot: 'little_red_dot',
  clumpy: 'clumpy_disk',
  tidal: 'tidal_feature',
  artifact: 'artifact',
  high_z: 'high_z_candidate',
};

export function CategoryIcon({ category, size = 20, labelled = false }: { category: string; size?: number; labelled?: boolean }) {
  const src = svgAsset('categories', FILE_BY_CATEGORY[category] ?? category);
  if (!src) return null;
  return <img src={src} alt={labelled ? categoryLabel(category) : ''} width={size} height={size} />;
}

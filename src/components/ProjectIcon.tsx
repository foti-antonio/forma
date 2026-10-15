import { Layers, Code2, Rocket, Box, Globe, Sparkles } from 'lucide-react';
import type { ProjectIcon as IconName } from '../lib/ipc';
const icons = {
  layers: Layers,
  code: Code2,
  rocket: Rocket,
  box: Box,
  globe: Globe,
  sparkles: Sparkles,
};
export function ProjectIcon({
  icon,
  color,
  small = false,
}: {
  icon: IconName;
  color: string;
  small?: boolean;
}) {
  const Icon = icons[icon] ?? Layers;
  return (
    <span
      className={small ? 'project-icon small' : 'project-icon'}
      style={{ color, backgroundColor: color + '18' }}
    >
      <Icon size={small ? 15 : 21} />
    </span>
  );
}

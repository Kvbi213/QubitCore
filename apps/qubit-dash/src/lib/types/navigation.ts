export type ActiveTab = 'overview' | 'providers' | 'routing' | 'routines' | 'security';

export interface NavItem {
  id: ActiveTab;
  number: string;
  label: string;
  badge?: string;
  badgeColor?: 'cyan' | 'mint' | 'purple' | 'amber';
}

export const NAV_ITEMS: NavItem[] = [
  { id: 'overview', number: '01', label: 'Overview' },
  { id: 'providers', number: '02', label: 'Providers & Auth', badge: 'LIVE', badgeColor: 'mint' },
  { id: 'routing', number: '03', label: 'Mode Routing' },
  { id: 'routines', number: '04', label: 'Routines', badge: '148', badgeColor: 'cyan' },
  { id: 'security', number: '05', label: 'Security & Storage' },
];

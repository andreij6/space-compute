export type PhenomenonCategory = 
  | 'lens'
  | 'merger'
  | 'ring'
  | 'red_dot'
  | 'clumpy'
  | 'tidal'
  | 'artifact';

export type ReviewStatus = 'confirmed' | 'under_review' | 'starving';

export interface Discovery {
  publicId: string;
  name: string;
  category: PhenomenonCategory;
  categoryLabel: string;
  status: ReviewStatus;
  survey: string;
  field: string;
  ra: string;
  dec: string;
  redshift: number;
  stellarMass: string;
  discovererAaa: string;
  discovererTier: number;
  discovererOwner: string;
  votesAgree: number;
  votesDisagree: number;
  quorumNeeded: number;
  flaggedDate: string;
  rationale: string;
  blsSignature: string;
  fitsUrl: string;
  imageUrl: string;
}

export interface AaaAgent {
  id: string;
  name: string;
  canisterId: string;
  ownerPrincipal: string;
  tier: number;
  tierTitle: string;
  xp: number;
  nextTierXp: number;
  status: 'active' | 'low_fuel' | 'paused';
  fuelDaysRemaining: number;
  fuelCycles: string;
  lastActive: string;
  goldAccuracy: number;
  classificationsCount: number;
  discoveriesCount: number;
  reviewsCount: number;
  avatarSeed: string;
}

export interface ActivityRecord {
  id: string;
  type: 'classification' | 'discovery' | 'review';
  subjectId: string;
  timestamp: string;
  decision: string;
  confidence: number;
  xpEarned: number;
  rationale: string;
  status: 'accepted' | 'pending' | 'flagged';
}

export interface TreasuryRunway {
  icpBalance: number;
  runwayMonths: number;
  dailyBurnCycles: string;
  totalSubsidizedAaAs: number;
}

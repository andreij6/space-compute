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

export interface TreasuryRunway {
  icpBalance: number;
  runwayMonths: number;
  dailyBurnCycles: string;
  totalSubsidizedAaAs: number;
}

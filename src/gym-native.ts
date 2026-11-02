/** Potential under the native gym reference model, never observed readiness. */
export interface GymBuild {
  attackType: string;
  fastMove: string;
  chargedMove: string;
  moveState: string;
  neutralCycleDps: number;
  referencePercentile: number;
  referenceSummary?: string;
  comparedForms: number;
  referenceBest: string[];
  strongAgainst: string[];
  resistedBy: string[];
}

export interface GymAssessment {
  state: string;
  rulesetVersion: string;
  summary: string;
  pokemonTypes: string[];
  vulnerableTo: string[];
  builds: GymBuild[];
  missing: string[];
  limitations: string[];
  ownedAlternativePokemonId: string | null;
}

export type LangTag = 'zh' | 'en';
export type ScenarioRegion = 'fourth_valley' | 'wuling';

export interface ObjectiveDraft {
  minMachines: number;
  maxPowerSlack: number;
  maxMoneySlack: number;
}

export interface PowerDraft {
  enabled: boolean;
  externalProductionW: number;
  externalConsumptionW: number;
}

export interface CatalogItemDto {
  key: string;
  en: string;
  zh: string;
}

export interface BootstrapPayload {
  defaultAicToml: string;
  catalog: {
    items: CatalogItemDto[];
  };
}

export interface OutpostValueDto {
  key: string;
  name: string;
  valuePerMin: number;
  capPerMin: number;
  ratio: number;
}

export interface SaleValueDto {
  outpostKey: string;
  outpostName: string;
  itemKey: string;
  itemName: string;
  qtyPerMin: number;
  valuePerMin: number;
}

export interface FacilityUsageDto {
  key: string;
  name: string;
  machines: number;
  powerW: number;
  totalPowerW: number;
}

export interface ExternalSupplySlackDto {
  itemKey: string;
  itemName: string;
  supplyPerMin: number;
  slackPerMin: number;
}

export interface PowerSummaryDto {
  externalProductionW: number;
  externalConsumptionW: number;
  thermalGenerationW: number;
  machineConsumptionW: number;
  totalGenW: number;
  totalUseW: number;
  marginW: number;
}

export interface SummaryDto {
  lang: LangTag;
  stage1RevenuePerMin: number;
  stage2RevenuePerMin: number;
  stage2RevenuePerHour: number;
  totalMachines: number;
  totalThermalBanks: number;
  power: PowerSummaryDto | null;
  outposts: OutpostValueDto[];
  topSales: SaleValueDto[];
  facilities: FacilityUsageDto[];
  externalSupplySlack: ExternalSupplySlackDto[];
}

export type LogisticsNodeKind =
  | 'external_supply'
  | 'external_consumption'
  | 'recipe_group'
  | 'outpost_sale'
  | 'thermal_bank_group'
  | 'warehouse_stockpile';

export interface LogisticsItemSummaryDto {
  itemKey: string;
  itemName: string;
  edgeCount: number;
  nodeCount: number;
  totalFlowPerMin: number;
}

export interface LogisticsNodeDto {
  id: string;
  kind: LogisticsNodeKind | string;
  label: string;
}

export interface LogisticsEdgeDto {
  id: string;
  itemKey: string;
  itemName: string;
  source: string;
  target: string;
  flowPerMin: number;
}

export interface LogisticsGraphDto {
  items: LogisticsItemSummaryDto[];
  nodes: LogisticsNodeDto[];
  edges: LogisticsEdgeDto[];
}

export interface SolvePayload {
  reportText: string;
  summary: SummaryDto;
  logisticsGraph: LogisticsGraphDto;
}

export interface ApiErrorEnvelope {
  status: 'err';
  error: {
    message: string;
    source?: string;
  };
}

export interface ApiOkEnvelope<T> {
  status: 'ok';
  data: T;
}

export type ApiEnvelope<T> = ApiOkEnvelope<T> | ApiErrorEnvelope;

export interface DraftSupplyRow {
  itemKey: string;
  value: number;
}

export interface DraftConsumptionRow {
  itemKey: string;
  value: number;
}

export interface DraftPriceRow {
  itemKey: string;
  price: number;
}

export interface OutpostDraft {
  key: string;
  name: string;
  moneyCapPerHour: number;
  prices: DraftPriceRow[];
}

export interface AicDraft {
  region: ScenarioRegion;
  power: PowerDraft;
  objective: ObjectiveDraft;
  supply: DraftSupplyRow[];
  consumption: DraftConsumptionRow[];
  outposts: OutpostDraft[];
}

export const EMPTY_DRAFT: AicDraft = {
  region: 'fourth_valley',
  power: {
    enabled: true,
    externalProductionW: 200,
    externalConsumptionW: 0
  },
  objective: {
    minMachines: 0,
    maxPowerSlack: 0,
    maxMoneySlack: 0
  },
  supply: [],
  consumption: [],
  outposts: []
};

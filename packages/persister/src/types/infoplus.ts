export type BooleanString = "J" | "N";

export const enum InfoStatus {
  Planned = "Gepland",
  Live = "Actueel",
}

export type InfoStatusAttr<T> = [
  T & { attr: { InfoStatus: InfoStatus.Planned } },
  (T & { attr: { InfoStatus: InfoStatus.Live } }) | undefined,
];

export const enum TrainStatus {
  Unknown = "0",
  Approaching = "1",
  ArrivingOrAtPlatform = "2",
  DoorsOpen = "3",
  DoorsClosed = "4",
  Departed = "5",
}

export type Change = {
  WijzigingType: ChangeType;
  WijzigingStation?: Station;
  PresentatieWijziging?: {
    Uitingen: {
      Uiting: {
        text: string;
        attr: {
          Prioriteit: string;
        };
      };
      attr: {
        Taal: string;
      };
    }[];
  };
};

export const enum ChangeType {
  DepartureDelayed = "10",
  ArrivalDelayed = "11",

  DepartureTimeChanged = "12",
  ArrivalTimeChanged = "13",

  DeparturePlatformChanged = "20",
  ArrivalPlatformChanged = "21",

  DeparturePlatformAllocated = "22",
  ArrivalPlatformAllocated = "23",

  StoppingBehaviourChanged = "30",

  AdditionalDeparture = "31",
  DepartureCancelled = "32",

  Diverted = "33",
  DestinationChangedShorterRoute = "34",
  DestinationChangedLongerRoute = "35",
  OriginChangedShorterRoute = "36",
  OriginChangedLongerRoute = "37",

  AdditionalArrival = "38",
  ArrivalCancelled = "39",

  DestinationChanged = "41",
  OriginChanged = "42",

  AdditionalPassage = "43",
  PassageCancelled = "44",

  // Niet actuele logische rit
  JourneyWithoutLiveInformation = "50",
  // Trein vervangend vervoer (TVV)
  RailReplacementService = "51",

  // Onlogisch materieel [voor een] Intercity
  IntercityOperatedWithSprinter = "80",
  // Onlogisch materieel [voor een] Sprinter
  SprinterOperatedWithIntercity = "81",
}

export type Station = {
  StationCode: string;
  Type: string;
  KorteNaam: string;
  MiddelNaam: string;
  LangeNaam: string;
  UICCode: string;
};

// ISO date string: YYYY-mm-dd
export type DateISOString = string;

export type DateTimeISOString = string;

// Time string: HH:mm:ss
export type TimeString = string;

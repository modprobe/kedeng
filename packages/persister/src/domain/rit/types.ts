import {
  type BooleanString,
  type Change,
  type DateTimeISOString,
  type InfoStatusAttr,
  type Station,
} from "../../types/infoplus";

type RollingStockPart = {
  MaterieelDeelID: string;
  MaterieelDeelSoort: string;
  MaterieelDeelAanduiding: string;
  MaterieelDeelLengte: string;
  MaterieelDeelVertrekPositie: string;
  MaterieelDeelVolgordeVertrek: string;
  MaterieelNummer: string;
  MaterieelDeelToegankelijk: BooleanString;
  MaterieelDeelEindBestemming: InfoStatusAttr<Station>;
  AchterBlijvenMaterieelDeel: BooleanString;
};

export const enum StopType {
  Stop = "X",
  Passage = "D",
}

export type JourneySegmentStation = {
  Station: Station;
  StationToegankelijk: BooleanString;
  StationReisAssistetie: BooleanString;
  TreinEindBestemming: InfoStatusAttr<Station>;
  Stopt: InfoStatusAttr<{
    text: BooleanString;
  }>;
  AankomstTijd?: InfoStatusAttr<{
    text: DateTimeISOString;
  }>;
  TreinAankomstSpoor?: InfoStatusAttr<{
    SpoorNummer: string;
    SpoorFase?: string;
  }>;
  VertrekTijd?: InfoStatusAttr<{
    text: DateTimeISOString;
  }>;
  TreinVertrekSpoor?: InfoStatusAttr<{
    SpoorNummer: string;
    SpoorFase?: string;
  }>;
  StationnementType: StopType;
  MaterieelDeel?: RollingStockPart[];
  NietInstappen: BooleanString;
  TreinRangeertVolledigAf: BooleanString;
  Wijziging?: Change[];
};

export type JourneySegment = {
  LogischeRitDeelNummer: string;
  LogischeRitDeelStation: JourneySegmentStation[];

  Wijziging?: Change[];
};

export type Journey = {
  LogischeRitNummer: string;
  LogischeRitDeel: JourneySegment[];
  Wijziging?: Change[];
};

export type RitMessage = {
  PutReisInformatieBoodschapIn: {
    ReisInformatieProductRitInfo: {
      RitInfo: {
        TreinNummer: string;
        TreinDatum: string;
        TreinSoort: {
          text: string;
          attr: {
            Code: string;
          };
        };

        Vervoerder: string;
        Reserveren: BooleanString;
        Toeslag: BooleanString;
        SpeciaalKaartje: BooleanString;
        Reisplanner: BooleanString;

        LogischeRit: Journey;
      };
    };
  };
};

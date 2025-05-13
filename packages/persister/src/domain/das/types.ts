import type {
  ChangeType,
  InfoStatusAttr,
  Station,
  TrainStatus,
} from "../../types/infoplus";

export type DasMessage = {
  PutReisInformatieBoodschapIn: {
    ReisInformatieProductDAS: {
      DynamischeAankomstStaat: {
        RitId: string;
        RitDatum: string;
        RitStation: Station;

        TreinAankomst: {
          TreinNummer: string;
          TreinSoort: {
            text: string;
            attr: {
              Code: string;
            };
          };

          TreinStatus: TrainStatus;
          LijnNummer?: string;
          Vervoerder: string;

          TreinHerkomst: InfoStatusAttr<Station>;

          AankomstTijd: InfoStatusAttr<{ text: string }>;
          ExacteAankomstVertraging: string;

          TreinAankomstSpoor?: InfoStatusAttr<{ SpoorNummer: string }>;

          WijzigingHerkomst?: {
            WijzigingType: ChangeType;
          };
        };
      };
    };
  };
};

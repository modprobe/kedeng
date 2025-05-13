import type {
  BooleanString,
  ChangeType,
  DateTimeISOString,
  InfoStatusAttr,
  Station,
  TrainStatus,
} from "../../types/infoplus";

export type DvsMessage = {
  PutReisInformatieBoodschapIn: {
    ReisInformatieProductDVS: {
      DynamischeVertrekStaat: {
        RitId: string;
        RitDatum: string;
        RitStation: Station;
        Trein: {
          TreinNummer: string;
          TreinSoort: {
            text: string;
            attr: {
              Code: string;
            };
          };
          TreinFormule: string;
          TreinStatus: TrainStatus;
          LijnNummer?: string;
          Vervoerder: string;

          Reserveren: BooleanString;
          Toeslag: BooleanString;
          NietInstappen: BooleanString;
          AchterBlijvenAchtersteTreinDeel: BooleanString;
          RangeerBeweging: BooleanString;
          SpeciaalKaartje: BooleanString;

          TreinEindBestemming: InfoStatusAttr<Station>;

          VertrekTijd: InfoStatusAttr<{
            text: DateTimeISOString;
          }>;

          ExacteVertrekVertraging: string;

          TreinVertrekSpoor?: InfoStatusAttr<{ SpoorNummer: string }>;

          Wijziging?: {
            WijzigingType: ChangeType;
          };
        };
      };
    };
  };
};

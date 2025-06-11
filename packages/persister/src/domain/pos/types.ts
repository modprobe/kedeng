import type { DateTimeISOString } from "../../types/infoplus";

export type PosMessage = {
  ArrayOfTreinLocation: {
    TreinLocation: {
      TreinNummer: string;
      TreinMaterieelDelen: {
        MaterieelDeelNummer: string;
        Materieelvolgnummer: string;
        GpsDatumTijd: DateTimeISOString;
        Latitude: string;
        Longitude: string;
        Snelheid: string;
        Richting: string;
        Elevation: string;
        AantalSatelieten: string;
      }[];
    }[];
  };
};

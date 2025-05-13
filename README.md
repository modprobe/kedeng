# kedeng

![Logo](./logo.jpg)

> Vanochtend vroeg vertrokken in de luwte van de nacht<br>
> En tien minuten op de trein gewacht<br>
> Want die had wat vertraging mijn God daar baal ik van

## Components

- **data-importer**
  - imports the necessary static data: timetables and stations
  - timetables:
    - are fetched automatically from [NDOV Loket](https://data.ndovloket.nl/ns/ns-latest.zip)
    - IFF format is parsed
  - station data is fetched from the [NS API](https://apiportal.ns.nl/api-details#api=nsapp-stations-api&operation=getStationsV3)

- **receiver**:
  - small layer that receives messages from the [NDOV Loket zeromq](https://data.ndovloket.nl/REALTIME.TXT) and pushes them into NATS streams so we have better control over the queue

- **persister**:
  - pulls from the NATS streams
  - processes the received messages
  - persists changes in the database
  - currently implemented:
    - [x] DVS (_Dynamische VertrekStaat_, departures)
    - [x] DAS (_Dynamische AankomstStaat_, arrivals)
    - [x] RIT (_Ritinformatie_, journeys)
    - [ ] POS (train positions)
    - [ ] PIL (_PatroonInformatie Landelijk_, aka LAB (_LAndelijke Bericht_), nation-wide disruptions)
    - [ ] PIS (_PatroonInformatie Station_, aka STB (_StationsBericht_), disruptions for stations)
    - [ ] VTB-L (_Vrije TekstBericht Landelijk_, free-text nation-wide disruptions)
    - [ ] VTB-S (_Vrije TekstBericht Station_, free-text disruptions for stations)

- **api**:
  TBD
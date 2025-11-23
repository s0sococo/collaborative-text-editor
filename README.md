# Coolaborative Text Editor

Praca inzynierska Sofya Karahoda, Piotr Jankiewicz 

Mini PW Informatyka i Systemy informatyczne, 2025/26

## Architekrura

Do działania projektu potrzebny jest uruchomiony serwer SFU od LiveKit. LiveKit 
to opensource serwer wykorzystywany do komunikacji real time z mozliwościa tworzenia
 pokoi/kanałów. Do odpalenia edytora potrzebne jest stworzenie pliku .env z odpowiednimi 
 stałymi zapewniającymi komunikację z serwerem. Struktura .env powinna być taka: 

 ```
LIVEKIT_API_KEY=key
LIVEKIT_API_SECRET=secret
LIVEKIT_URL=ipaddress:port
 ```

dla lokalnego serwera wygląda to tak: 

```
LIVEKIT_API_KEY=devkey
LIVEKIT_API_SECRET=devsecret
LIVEKIT_URL=127.0.0.1:7880
```

Aby uruchomić edytor trzeba skorzystac z flagi przy uruchamianiu kompilatora by 
uzyc odpowiedniego kompliatora obiektowego c.

```
~ RUSTFLAGS="-C link-arg=-ObjC" cargo run
```

### Jak uruchomić lokalny serwer SFU live kit i jak z niego korzystać w command line?

Aby uruchomić lokalny serwer trzeba mieć zainstalowany docker.

```
~ docker run -d \
-p 7880:7880 \
-p 7881:7881/udp \
-e "LIVEKIT_KEYS=devkey: devsecret" \
-e LIVEKIT_LOG_LEVEL=debug \
livekit/livekit-server \
--node-ip 127.0.0.1 \
--dev
~
```

W ten sposób uruchamiany jest serwer `livekit/livekit-server` który na naszej maszynie 
pod adresem 127.0.0.1:7800 ma uruchomione swoje API. Aby potwierdzić, ze serwer jest
 uruchomiony i responsywny mozna wykonać zapytanie: 

 ```
 ~ curl http://127.0.0.1:7880
 OK%
 ~
 ```

Ponaddto mozna podejrzec logi maszyny w nastepujacy sposob: 

```
~ docker ps 
CONTAINER ID   IMAGE                    COMMAND                  ...
abcd1234   livekit/livekit-server   "/livekit-server --n…"   ....
~  docker logs abcd1234

...LOGI...

```

#### Jak korzystać z CLI LiveKIT ?

Najpierw trzeba mieć ten command line zaintalowany. Na stacji roboczej macos 
mozna zainstalowac go w następujący sposób:

```
~ brew install livekit-cli
```

Aby wskazać projekt/serwer z którym command line móglby się łaczyć nalezy wykonać 
takie komendy: 

```
~  lk project add --api-key <key> --api-secret <secret> <project_name>
```

czyli dla połaczenia lokalnego nalezy wykonac: 

```
~ lk project add --api-key devkey --api-secret devsecret inzynierka_local
```

Wynikiem tej koncifguracji CLI jest: 

```
Saved CLI config to [/Users/-----/.livekit/cli-config.yaml]
┌────────────────────┬─────────────────────────────────────────┬─────────────────┐
│ Name               │ URL                                     │ API Key         │
├────────────────────┼─────────────────────────────────────────┼─────────────────┤
│   inzynierka_local │ ws://127.0.0.1:7880                     │ devkey          │
└────────────────────┴─────────────────────────────────────────┴─────────────────┘
```

Co nam to daje ? Najwazniejsze mozliwosci to podglad utworzonych kanałow i liczbe ich uzytkowników, ale CLI daje mozliwosc tworzenia nowych pokoi, access_tokenów do łączenia się z kanałami przez uzytkownikow. 

```
~ lk room list
Using project [local-inzynierka]
┌────────┬──────┬──────────────┬────────────┐
│ RoomID │ Name │ Participants │ Publishers │
├────────┼──────┼──────────────┼────────────┤
└────────┴──────┴──────────────┴────────────┘
```

Po więcej koment odsyłam do dokumentacji lib `lk --help`.


## Edytor

Aktualnie edytor umozliwia tworzenie pokoju lub dolaczenie do pokoju. Logika synchronizacji danych jeszcze nie została zaimplementowana. 
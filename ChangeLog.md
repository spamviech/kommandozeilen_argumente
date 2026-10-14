# Changelog for kommandozeilen_argumente

## Unreleased changes

## 0.3.0

### Breaking changes

- Die englische API ist nun primär: Neue Anwendungen sollen `Arguments`, `Description`, `Value`,
  `EarlyExit`, `Result` und `Language` verwenden. Die deutschen Namen bleiben als explizite
  Spiegeltypen beziehungsweise Kompatibilitäts-APIs erhalten.
- Die Modulstruktur verwendet englische Primärnamen (`arguments`, `description`, `outcome`,
  `language`); `arguments::argumente` stellt den deutschen `Argumente`-Spiegel bereit.
- `combine!` ist das bevorzugte Makro. `kombiniere!` bleibt als Kompatibilitäts-Wrapper für
  `Argumente` verfügbar.
- Das Feature `derive` gehört nicht mehr zur Standard-Feature-Liste und muss bei Bedarf explizit
  aktiviert werden.

### Parsing

- Der Parser ist vollständig als sechs-stufige Pipeline implementiert: zusammengefasste
  Kurzformen, eigenständige Kurzformen und Langformen werden vor der typisierten Auswertung
  erkannt; anschließend werden Alternativen validiert, ausgewählt und akkumuliert.
- Zusammengefasste Kurzformen haben Vorrang. Verbleibende Teile eines zusammengefassten Blocks
  werden nicht als eigenständige Namen oder Werte wiederverwendet.
- Werte bleiben bis zu ihrem besitzenden `Value` als `OsString` erhalten. Alternativen werden in
  Reihenfolge geprüft; ein Fehler beim Parsen eines Werts verwirft nur die betroffene Alternative.
- Parserdefinitionen und Callbacks müssen für die Alternativenauswahl nicht `Clone` implementieren.

### API and maintenance

- Neue Typen `Wert`, `Flag`, `FrühesBeenden` und `EinzelArgument` mit `From`-Implementierungen
  für Argumente; alle Felder sind `pub`.
- `Alternativen` für Argumente hinzugefügt.
- `Normalisiert::as_str` hinzugefügt.
- Clippy-Lints bereinigt und `venial` für das Parsing der Proc-Makro-Attribute verwendet.

## 0.2.0

- Normalisiere Namen und Prä-/Infixe, bevor diese mit Nutzereingaben verglichen werden.
- Erlaube von Groß-/Kleinschreibung unabhängige Namen und Prä-/Infixe.
- Erlaube Konfiguration aller Prä-/Infixe.
- Unbenutzten `&self`-Parameter von folgenden Methoden entfernt:
  - `Parse::parse_vollständig_mit_sprache`
  - `Parse::parse_vollständig_mit_sprache_aus_env`
- Funktion ersetzt: `unicase_eq` -> `unicode_eq`
    Vor Vergleich beider Strings werden beide in Unicode Normalization Form C konvertiert.
    Beachten der Groß-/Kleinschreibung wird über einen Parameter gesteuert.
- Methode umbenannt: `Ergebnis::map` -> `Ergebnis::konvertiere`, `Ergebnis::convert`
- Die `kombiniereN`-Funktionen werden explizit (über ein Macro) implementiert.
    Dadurch werden deutlich weniger Tupel ge- und entpackt,
    wie es bei der `kombiniere!`-basierten Implementierung noch der Fall war.
- Das `kombiniere!`-Macro erzeugt und entpackt deutlich weniger Tupel als bisher.
    Sofern möglich wird die effizientere Implementierung über die `kombiniereN`-Funktionen verwendet.
- Bereitstellen eines `Konfiguration`-Typen und einer `konfigurationen`-Methode.
    Damit kann z.B. ein eigener Hilfe-Text erzeugt werden.
- Stelle englische Synonyme für Typen, Macros, Funktionen und Methoden bereit.
- kommandozeilen_argumente_derive verwendet syn-feature "derive" statt "full".
- kommandozeilen_argumente_derive parsed Attribute direkt aus einem TokenStream.

## 0.1.3

- Durch kommandozeilen_argumente_derive erzeugte [Parse](https://docs.rs/kommandozeilen_argumente/latest/kommandozeilen_argumente/trait.Parse.html)-Instanzen
    verwendet erneut die `=>` Syntax für das [kombiniere!](https://docs.rs/kommandozeilen_argumente/latest/kommandozeilen_argumente/macro.kombiniere.html) Macro.
- `funktion`-Beispiel an die neue Syntax angepasst.

## 0.1.2

- Das [kombiniere!](https://docs.rs/kommandozeilen_argumente/latest/kommandozeilen_argumente/macro.kombiniere.html)-Macro
    akzeptiert eine Komma-Liste als Argumente.
    Die neue bevorzugte Syntax ist `kombiniere!(funktion, <args>)`
    und kann mit `rustfmt` formatiert werden.
    Die bisherige Syntax mit `kombiniere!(funktion => <args>)` wird weiterhin akzeptiert.
- Durch kommandozeilen_argumente_derive erzeugte [Parse](https://docs.rs/kommandozeilen_argumente/latest/kommandozeilen_argumente/trait.Parse.html)-Instanzen
    verwenden die neue kombiniere!-Syntax.

## 0.1.1

- Wert-Argumente, die aus Strings geparst werden (Argumente::wert_string*).

## 0.1.0

Erster Release

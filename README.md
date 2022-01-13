# kommandozeilen_argumente

Parser für Kommandozeilen-Argumente mit optionaler, automatischer Hilfe-Generierung.

Zum erstellen eines neuen Arguments werden assoziierte Funktionen bereitgestellt,
der Ergebnistyp wird dabei durch eine Typ-Variable festgelegt.
Argumente können mithilfe des `kombiniere!`-Macros, bzw. dedizierten `kombiniereN`-Funktionen,
zu komplexeren Strukturen zusammengefasst werden,
die zum parsen potentiell mehrere Argumente benötigen.

Ein Argument wird durch seinen Langnamen oder potentiellen Kurznamen identifiziert.
Angabe eines Langnamens startet mit zwei Minus `--lang`.
Angabe eines Kurznamens startet mit einem Minus `-k`.
Für Kurznamen wird angenommen, dass sie nur ein [Grapheme](https://docs.rs/unicode-segmentation/1.8.0/unicode_segmentation/trait.UnicodeSegmentation.html#tymethod.graphemes) lang sind.

Alle verwendeten Strings, z.B. für die erzeugte Hilfe-Meldung, sind konfigurierbar.
Sofern es relevant ist werden für Deutsch und Englisch spezialisierte Funktionen bereitgestellt.

Argumente können Standard-Werte haben, der verwendet wird sofern keiner ihrer Namen verwendet wird.
Ohne Standard-Wert muss das Argument verwendet werden, ansonsten schlägt das Parsen fehl.

## Flags

Flags sind Argumente ohne Wert, sie können entweder aktiviert oder deaktiviert sein.
Meistens repräsentieren sie `bool`-Argumente, andere Typen werden aber ebenfalls unterstützt.

Angenommen Langnamen `flag`, Kurznamen `f` und invertiere_präfix `kein`,
eine Flag kann mit `--flag` oder `-f` aktiviert
und mit `--kein-flag` deaktiviert werden.

Existieren mehrere Flags mit Kurznamen `f`, `g` und `h`,
so können alle gleichzeitig mit `-fgh` aktiviert werden.

### Frühes Beenden

Eine besondere Art von Flags führt zu frühem beenden.
Sie können nicht deaktiviert werden und führen zu vorzeitigem Beenden unter Anzeigen einer Nachricht.
Typische Anwendungsfälle sind Anzeigen der aktuellen Version oder des Hilfe-Textes.

## Werte

Argumente können ebenfalls Werte spezifizieren.
Bei Langnamen wird der Wert getrennt von einem Leerzeichen, oder `=`-Zeichen angegeben.
Bei Kurznamen kann der Wert auch direkt im Anschluss an den Namen angegeben werden.
Die Konvertierung des Wertes wird aus `&OsStr` versucht.

Es ist möglich, alle erlaubten Werte im Hilfe-Text anzeigen zu lassen.

Angenommen Langnamen `--wert` und Kurznamen `-w` für ein Zahlen-Argument,
`--wert 3`, `--wert=3`, `-w 3`, `-w=3` und `-w3` werden alle mit Ergebnis `3` geparst.

## Feature derive

Mit aktiviertem `derive`-Feature können die akzeptieren Kommandozeilen-Argumente
automatisch erzeugt werden.
Dazu wird das `Parse`-Trait für ein `struct` implementiert.
Normalerweise wird das `ParseArgument`-Trait zum parsen verwendet.
Es ist implementiert für `String`, Zahlentypen (`i8`, `u8`, `i16`, `u16`, ..., `f32`, `f64`),
`Option<T>` und Typen, die das `ArgEnum`-Trait implementieren.
Das `ArgEnum`-Trait kann automatisch für ein `enum`, das keine Daten hält abgeleitet werden.

TODO

trait ArgEnum ohne Attribute, derive-Macro wird für enums ohne Daten bereitgestellt.

trait Parse: alles über kommandozeilen_argumente-Attribut

Global an struct

- `sprache: <sprache>` | `language: <language>`:
  Standard-Einstellung für einige Strings, Standard: `english`.
  Vorgefertigte Sprachen für `deutsch`, `englisch` und `english`.
- `version`: erzeuge eine `--version` Flag
- `hilfe` | `help`: erzeuge eine Hilfe-Text
- `hilfe(<opts>)`, `help(<opts>)`, `version(<opts>)`:
  Wie die Variante ohne opts, nur Kurzname ist standardmäßig deaktiviert. Mögliche Opts:
  - `lang: <name>`, `long [<namen>]`: Setze Langnamen explizit
  - `kurz`: Setze Kurznamen als erstes Grapheme des originalen Langnamen
  - `kurz: <name>`, `kurz: [<namen>]`: Setze Kurznamen explizit
  - `sprache: <sprache>` | `language: <language>`: Sprache von Hilfe-Text und Standard-Namen
- `meta_var: <string>` | `meta_var: <string>`:
  Setze Standardwert für in der Hilfe angezeigte Meta-Variable, Standard `WERT` oder `VALUE`
- `invertiere_präfix: <string>` | `invert_prefix: <string>`:
  Setze Standardwert für Präfix zum invertieren einer Flag, Standard `kein` oder `no`

Vor Feldern

- hilfe aus docstring
- name wird langer name
- erstes grapheme wird kurzer name
- Es wird eine Implementierung über das ParseArgument-Trait verwendet
  - Flags (bool-Werte) sind standardmäßig deaktiviert
  - Option-Werte sind standardmäßig None
  - Strings und Zahlentypen (u8, i8, f32, ...) haben keinen Standardwert (notwendiges Argument)
  - ArgEnum-Werte haben keinen Standardwert (notwendiges Argument), benötigt Display.
    Derive-Macro für Summen-Typen ohne Daten wird bereitgestellt.
- `glätten`/`flatten`: verwende das Parse-Trait (übernehmen der konfigurierten Argumente)
- `FromStr`: verwende das FromStr-Trait (benötigt Display für Wert und Fehler-Typ)
- `benötigt`/`required`: entferne den konfigurierten Standard-Wert
- `lang: <name>`| `long: <name>`: bestimme Langname explizit
- `lang: [<namen>]` | `long: [<names>]`: bestimme Langnamen explizit (Komma-getrennte Liste)
- `kurz`/`short`: Verwende eine Kurzform, bestehend aus dem ersten Grapheme der Langform
- `kurz: <wert>"`/`short: <value>"`: Verwende die spezifizierte Kurzform
- `kurz: [<namen>]` | `short: [<names>]`: bestimme Kurzformen explizit (Komma-getrennte Liste)
- `standard: <wert>` | `default: <value>`: setzte den Standard-Wert
- `meta_var: <string>`: setzte die in der Hilfe angezeigt Meta-Variable
- `invertiere_präfix: <string>` | `invert_prefix: <string>`: setze Präfix zum invertieren einer Flag

## Geplante Features

- Unterargumente (subcommands)
- Positions-basierte Argumente
- Unterschiedlicher Standard-Wert für Name ohne Wert und Name kommt nicht vor
- Argumente unter Berücksichtigung von Groß- und Kleinschreibung
- Argument-Gruppen (nur eine dieser N Flags kann gleichzeitig aktiv sein)

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

## Feature "derive"

Mit aktiviertem `derive`-Feature können die akzeptieren Kommandozeilen-Argumente
automatisch erzeugt werden.
Dazu wird das `Parse`-Trait für ein `struct` mit benannten Feldern implementiert.

Die Namen werden aus den Feld-Namen erzeugt,
der Langname ist der vollständige Feldname,
der Kurzname das erste Grapheme des Feldnamens.

Als Beschreibung im erzeugten Hilfe-Text wird der docstring des jeweiligen Feldes verwendet.

Zum parsen wird das `ParseArgument`-Trait verwendet.
Es ist implementiert für `bool`, `String`, Zahlentypen (`i8`, `u8`, `i16`, `u16`, ..., `f32`, `f64`),
`Option<T>` und Typen, die das `EnumArgument`-Trait implementieren.
Flag-Argumente werden für `bool`-Argumente erzeugt; diese sind standardmäßig deaktiviert.
Alle anderen Implementierungen erzeugen Wert-Argumente; `Option<T>` sind standardmäßig `None`,
alle anderen sind benötigte Argumente.
Das `EnumArgument`-Trait kann automatisch für ein `enum`, das keine Daten hält abgeleitet werden.
Für eine Verwendung als `ParseArgument` wird zusätzlich eine `Display`-Implementierung benötigt.

Das Standard-Verhalten kann über `#[kommandozeilen_argumente(<Optionen>)]`-Attribute beeinflusst werden.

Direkt am `struct` werden folgende Optionen unterstützt:

- `sprache: <sprache>` | `language: <language>`:
  Standard-Einstellung für einige Strings, Standard: `english`.
  Vorgefertigte Sprachen für `deutsch`, `englisch` und `english`.
- `version`: erzeuge eine `--version` Flag.
- `hilfe` | `help`: erzeuge eine Hilfe-Text.
- `hilfe(<opts>)`, `help(<opts>)`, `version(<opts>)`:
  Wie die Variante ohne opts, nur Kurzname ist standardmäßig deaktiviert. Mögliche Opts:
  - `lang: <name>`, `long [<namen>]`: Setze Langnamen explizit.
  - `kurz`: Setze Kurznamen als erstes Grapheme des originalen Langnamen.
  - `kurz: <name>`, `kurz: [<namen>]`: Setze Kurznamen explizit.
  - `sprache: <sprache>` | `language: <language>`: Sprache von Hilfe-Text und Standard-Namen.
- `meta_var: <string>` | `meta_var: <string>`:
  Setze Standardwert für in der Hilfe angezeigte Meta-Variable, Standard `WERT` oder `VALUE`.
- `invertiere_präfix: <string>` | `invert_prefix: <string>`:
  Setze Standardwert für Präfix zum invertieren einer Flag, Standard `kein` oder `no`.

Vor Feldern werden folgende Optionen unterstützt:

- `glätten`/`flatten`: verwende das Parse-Trait (übernehmen der konfigurierten Argumente).
- `FromStr`: verwende das FromStr-Trait (benötigt Display für Wert und Fehler-Typ).
- `benötigt`/`required`: entferne den konfigurierten Standard-Wert.
- `lang: <name>`| `long: <name>`: bestimme Langname explizit.
- `lang: [<namen>]` | `long: [<names>]`: bestimme Langnamen explizit (Komma-getrennte Liste).
- `kurz`/`short`: Verwende eine Kurzform, bestehend aus dem ersten Grapheme der Langform.
- `kurz: <wert>"`/`short: <value>"`: Verwende die spezifizierte Kurzform.
- `kurz: [<namen>]` | `short: [<names>]`: bestimme Kurzformen explizit (Komma-getrennte Liste).
- `standard: <wert>` | `default: <value>`: setzte den Standard-Wert.
- `meta_var: <string>`: setzte die in der Hilfe angezeigt Meta-Variable.
- `invertiere_präfix: <string>` | `invert_prefix: <string>`: setze Präfix zum invertieren einer Flag.

## Beispiel

Ein einfaches `struct` mit 3 Flags und 2 Werten, teilweise mit Standard-Werten kann über
folgenden Code ausgelesen werden:

```rust
use std::{
    fmt::{Debug, Display},
    num::NonZeroI32,
};

use kommandozeilen_argumente::{
    crate_name, crate_version, kombiniere, unicase_eq, Argumente, Beschreibung, EnumArgument,
    NonEmpty, ParseArgument, ParseFehler, Sprache,
};

#[derive(Debug, Clone)]
enum Aufzählung {
    Eins,
    Zwei,
    Drei,
}

impl EnumArgument for Aufzählung {
    fn varianten() -> Vec<Self> {
        use Aufzählung::*;
        vec![Eins, Zwei, Drei]
    }

    fn parse_enum(
        arg: &std::ffi::OsStr,
    ) -> Result<Self, kommandozeilen_argumente::ParseFehler<String>> {
        use Aufzählung::*;
        if let Some(string) = arg.to_str() {
            if unicase_eq(string, "Eins") {
                Ok(Eins)
            } else if unicase_eq(string, "Zwei") {
                Ok(Zwei)
            } else if unicase_eq(string, "Drei") {
                Ok(Drei)
            } else {
                Err(ParseFehler::ParseFehler(format!("Unbekannte Variante: {}", string)))
            }
        } else {
            Err(ParseFehler::InvaliderString(arg.to_owned()))
        }
    }
}

impl Display for Aufzählung {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

#[derive(Debug)]
struct Args {
    flag: bool,
    umbenannt: bool,
    benötigt: bool,
    wert: String,
    aufzählung: Aufzählung,
}

impl Display for Args {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Args { flag, umbenannt, benötigt, wert, aufzählung } = self;
        write!(f, "flag: {flag}\n")?;
        write!(f, "umbenannt: {umbenannt}\n")?;
        write!(f, "benötigt: {benötigt}\n")?;
        write!(f, "wert: {wert}\n")?;
        write!(f, "aufzählung: {aufzählung}\n")
    }
}

fn main() {
    let sprache = Sprache::DEUTSCH;
    let flag = Argumente::flag_bool_mit_sprache(
        Beschreibung::neu(
            "flag",
            None,
            Some("Eine Flag mit Standard-Einstellungen".to_owned()),
            Some(false),
        ),
        sprache,
    );
    let umbenannt = Argumente::flag_bool_mit_sprache(
        Beschreibung::neu(
            NonEmpty { head: "andere".to_owned(), tail: vec!["namen".to_owned()] },
            "u",
            Some("Eine Flag mit Standard-Einstellungen".to_owned()),
            Some(false),
        ),
        sprache,
    );
    let benötigt = Argumente::flag_bool(
        Beschreibung::neu(
            "benötigt",
            "b",
            Some(
                "Eine Flag ohne Standard-Wert mit alternativem Präfix zum invertieren.".to_owned(),
            ),
            None,
        ),
        "no",
    );
    let wert = String::argumente_mit_sprache(
        Beschreibung::neu("wert", None, Some("Ein String-Wert.".to_owned()), None),
        sprache,
    );
    let aufzählung = Argumente::wert_enum_display_mit_sprache(
        Beschreibung::neu(
            "aufzählung",
            "a",
            Some("Ein Aufzählung-Wert mit Standard-Wert.".to_owned()),
            Some(Aufzählung::Zwei),
        ),
        sprache,
    );
    let zusammenfassen = |flag, umbenannt, benötigt, wert, aufzählung| Args {
        flag,
        umbenannt,
        benötigt,
        wert,
        aufzählung,
    };
    let argumente = kombiniere!(zusammenfassen => flag, umbenannt, benötigt, wert, aufzählung)
        .hilfe_und_version_mit_sprache(crate_name!(), crate_version!(), sprache);
    let args = argumente
        .parse_vollständig_mit_sprache_aus_env(NonZeroI32::new(1).expect("1 != 0"), sprache);
    do_stuff("{:?}", args)
}
```

Mit aktiviertem `derive`-Feature ist ein identisches Verhalten mit folgendem Code möglich:

```rust
use std::{
    fmt::{Debug, Display},
    num::NonZeroI32,
};

use kommandozeilen_argumente::{EnumArgument, Parse};

#[derive(Debug, Clone, EnumArgument)]
enum Aufzählung {
    Eins,
    Zwei,
    Drei,
}

impl Display for Aufzählung {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

#[derive(Debug, Parse)]
#[kommandozeilen_argumente(hilfe, version, sprache: deutsch)]
struct Args {
    flag: bool,
    #[kommandozeilen_argumente(lang: [andere, namen], kurz: u)]
    umbenannt: bool,
    #[kommandozeilen_argumente(benötigt, kurz, invertiere_präfix: no)]
    benötigt: bool,
    wert: String,
    #[kommandozeilen_argumente(kurz, standard: Aufzählung::Zwei)]
    aufzählung: Aufzählung,
}

fn main() {
    let args = Args::parse_mit_fehlermeldung_aus_env(NonZeroI32::new(1).expect("1 != 0"));
    do_stuff(args)
}
```

In beiden Fällen wird folgender Hilfe-Text erzeugt:

```cmd
kommandozeilen_argumente 0.1.0

derive.exe [OPTIONEN]

OPTIONEN:
  --[kein]-flag                          Eine Flag mit 
Standard-Einstellungen. [Standard: false]
  --[kein]-(andere|namen) | -u           Eine Flag mit 
alternativen Namen. [Standard: false]
  --[no]-benötigt         | -b           Eine Flag ohne Standard-Wert mit alternativem Präfix zum invertieren.  --wert(=| )WERT                        Ein String-Wert.
  --aufzählung(=| )WERT   | -a[=| ]WERT  Ein Aufzählung-Wert mit Standard-Wert. [Erlaubte Werte: Eins, Zwei, Drei | Standard: Zwei]
  --version               | -v           Zeige die aktuelle Version an.
  --hilfe                 | -h           Zeige diesen Text an.
```

## (Noch) Fehlende Features

- Unterargumente (subcommands)
- Positions-basierte Argumente
- Unterschiedlicher Standard-Wert für Name ohne Wert und Name kommt nicht vor
- Argumente unter Berücksichtigung von Groß- und Kleinschreibung
- Argument-Gruppen (nur eine dieser N Flags kann gleichzeitig aktiv sein)

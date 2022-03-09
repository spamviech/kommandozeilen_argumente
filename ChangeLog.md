# Changelog for kommandozeilen_argumente

## Unreleased changes

- Die `kombiniereN`-Funktionen werden explizit (über ein Macro) implementiert.
    Dadurch werden deutlich weniger Tupel ge- und entpackt,
    wie es bei der `kombiniere!`-basierten Implementierung noch der Fall war.
- Das `kombiniere!`-Macro erzeugt und entpackt deutlich weniger Tupel als bisher.
    Sofern möglich wird die effizientere Implementierung über die `kombiniereN`-Funktionen verwendet.
- Stelle englische Synonyme für Typen, Macros, Funktionen und Methoden bereit.
- kommandozeilen_argumente_derive verwendet syn-feature "derive" statt "full".
- kommandozeilen_argumente_derive parsed Attribute direkt als TokenStream.

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

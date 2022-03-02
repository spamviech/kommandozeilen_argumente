# Changelog for kommandozeilen_argumente

## Unreleased changes

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

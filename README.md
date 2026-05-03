# HULKForge — Compilador del Lenguaje HULK

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-Academic-blue.svg)](#licencia)
![Status](https://img.shields.io/badge/status-In%20Development-yellow.svg)

**HULKForge** es la implementación de un compilador para **HULK** (Havana University Language Kompilation).

Esta rama contiene la primera fase: el **Lexer (análisis léxico)**.

---

## Instalación de dependencias

### Requisitos previos
- **Rust 1.70+** ([descargar](https://rustup.rs/))
- **Cargo** (incluido con Rust)

### Pasos iniciales

Después de clonar el repositorio, ejecuta:

```bash
# Descargar e instalar las dependencias del proyecto
cargo update

# Compilar en modo debug
cargo build

# Compilar en modo release (optimizado)
cargo build --release
```

---

## Descripción del Lexer

El lexer es la primera etapa del compilador. Convierte el código fuente en **tokens** (unidades léxicas).

### Características principales

- **Maximal Munch**: Reconoce los tokens más largos posibles (ej: `@@` como un solo token)
- **Posiciones precisas**: Cada token incluye línea y columna (base-1) para reportes de error puntuales
- **Recuperación de errores**: Continúa tokenizando tras encontrar caracteres inválidos, permitiendo reportar todos los errores a la vez
- **Números como strings**: Almacena números como `String` para permitir validación precisa en el parser (overflow detection, rango, formato)
- **Identificadores internos**: Soporta `_ident` para código generado internamente por el compilador en transpilaciones
- **Escapes en strings**: Soporta secuencias de escape estándar: `\"`, `\n`, `\t`, `\\`
- **Comentarios de línea**: Ignora automáticamente comentarios (`// ...`) sin generar tokens
- **Manejo de EOF**: Garantiza que el último token siempre es `Eof`, facilitando el uso en el parser
- **Velocidad**: Logos genera un lexer altamente optimizado basado en DFAs compilados
- **Confiabilidad**: El lexer evita panics explícitos y reporta errores mediante validación

### Tokens soportados

- **Palabras clave**: `let`, `if`, `function`, `type`, `while`, etc.
- **Operadores**: `+`, `-`, `*`, `/`, `@` (concatenación), `@@` (con espacio), `:=`, `=>`, etc.
- **Literales**: números (`"3.14"` como string), strings con escapes (`\"`, `\n`, `\t`, `\\`)
- **Identificadores**: `x`, `camelCase`, `snake_case`, etc.
- **Puntuación**: `(`, `)`, `{`, `}`, `[`, `]`, `;`, `,`, etc.

### Ejemplo de tokenización

**Código fuente:**

```hulk
let x = 42 in print(x);
```

**Salida léxica:**

```
Let                    (línea 1, col 1)
Ident("x")             (línea 1, col 5)
Eq                     (línea 1, col 7)
Number("42")           (línea 1, col 9)
In                     (línea 1, col 12)
Ident("print")         (línea 1, col 15)
LParen                 (línea 1, col 20)
Ident("x")             (línea 1, col 21)
RParen                 (línea 1, col 22)
Semicolon              (línea 1, col 23)
Eof                    (línea 1, col 24)
```

**Ejemplo con error:**

```hulk
let x = #42;
```

**Salida:**

```
Let
Ident("x")
Eq
[LexError] carácter inesperado: '#'  (línea 1, col 9)
Number("42")    // El lexer continúa tras el error
Semicolon
Eof
```

### Estructura actual

La estructura del proyecto en esta fase es:

```
src/
├── main.rs              # Punto de entrada (vacío)
└── lexer/
    ├── mod.rs           # Declaración del módulo lexer
    ├── lexer.rs         # Implementación del lexer
    └── test.rs          # Suite de tests
```

**Nota**: Esta estructura evolucionará con el parser (`parser/`), type-checker (`semantic/`), y generador de código (`codegen/`).

---

## Ejecutar los tests

```bash
# Ejecutar todos los tests
cargo test

# Ejecutar un test específico por nombre
cargo test keywords

# Ejecutar tests en modo verbose
cargo test -- --nocapture

# Ejecutar tests sin paralelización
cargo test -- --test-threads=1
```

### Cobertura de tests

La suite incluye amplia cobertura unitaria:
- ✅ Palabras clave del lenguaje
- ✅ Operadores de uno y múltiples caracteres
- ✅ Números enteros y flotantes
- ✅ Strings con secuencias de escape
- ✅ Identificadores e identificadores internos (`_ident`)
- ✅ Posicionamiento (línea/columna)
- ✅ Manejo de errores y recuperación
- ✅ Comentarios (`//`)
- ✅ EOF (fin de archivo)
- ✅ Programas completos

---

## Compilación

```bash
# Modo debug (rápido de compilar, lento de ejecutar)
cargo build

# Modo release (lento de compilar, rápido de ejecutar)
cargo build --release

# Ejecutar el binario compilado
./target/debug/hulk_forge      # Debug
./target/release/hulk_forge    # Release
```

---

## Dependencias

| Crate | Versión | Propósito |
|-------|---------|-----------|
| `logos` | 0.14 | Lexer declarativo basado en regex |
| `thiserror` | 2 | Tipos de error con Display automático |
| `miette` | 7 | Reportes de error con fancy output |
| `indexmap` | 2 | HashMap que preserva orden de inserción |

---

## Limitaciones actuales

Esta implementación del lexer establece el alcance deliberadamente en los siguientes aspectos:

- ❌ **Sin comentarios multilínea** — Sólo soporta `//` de una línea
- ❌ **Sin strings multilínea** — Los strings deben estar en una sola línea
- ❌ **Parser no implementado** — El siguiente paso es construir el AST
- ❌ **Identificadores internos (\_ident) requieren cuidado** — Solo válidos en código generado por el compilador, no en código de usuario
- ❌ **Sin números en otras bases** — Solo decimal está soportado (hex, octal, binario son futuros)
- ❌ **Sin tokens customizados en usuario** — El token enum es fijo

Estas limitaciones se abordarán en fases posteriores por diseño.

---

## Logos: Automatización de los 6 pasos del Lexer

### El proceso tradicional (manual)

Construir un lexer desde cero implica 6 pasos complejos:

1. **Definir reglas con expresiones regulares** — escribir patrones para cada token
2. **Convertir a NFAs (Thompson)** — transformar regexes en autómatas finitos no deterministas
3. **Pasar a DFA (Subset Construction)** — eliminar no-determinismo para mayor eficiencia
4. **Minimizar el DFA** — reducir estados innecesarios
5. **Generar código** — crear tablas de transiciones y funciones de reconocimiento
6. **Integrar en un bucle** — conectar con el parser para producir tokens

### Cómo Logos lo automatiza

**Logos** elimina pasos 2–5 automáticamente. Solo necesitas:

```rust
#[derive(Logos)]
pub enum Token {
    #[token("let")]          Let,
    #[regex(r"[0-9]+")]      Number(String),
    #[regex(r"[a-zA-Z]+")]   Ident(String),
    // ...
}
```

Logos genera internamente el DFA optimizado, compilado en código máquina. En tiempo de ejecución, procesa el fuente en **un solo pase**, directamente sin tablas interpretadas.

### Ventajas en HULKForge

- ✅ **Seguridad**: Compilación de tipos en Rust
- ✅ **Velocidad**: DFA compilado, no interpretado
- ✅ **Mantenibilidad**: Reglas legibles en el código
- ✅ **Precisión**: `logos` maneja maximal munch correctamente
- ✅ **Sin boilerplate**: No escribimos tablas de transiciones manualmente

---

## Contribuciones

Este proyecto es parte del curso de Compiladores en la Universidad de La Habana.

---

## Licencia

Proyecto académico © 2026

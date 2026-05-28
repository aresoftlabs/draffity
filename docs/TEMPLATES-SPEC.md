# Templates Spec

Las plantillas de Draffity son archivos JSON que describen una **estructura inicial** de proyecto (folders + capítulos seed) y, opcionalmente, un conjunto de **metadatos pedidos al usuario** al crear el proyecto.

Las plantillas built-in viven en [`packages/templates/`](../packages/templates/) y se embeben en el binario via `include_str!`. En premium, el loader podrá descubrir además plantillas en `~/.draffity/templates/` y, posteriormente, fetched de cloud.

## Versionado

El campo `schemaVersion` permite evolución. La versión actual es **1**. Cualquier valor distinto al esperado se rechaza al cargar.

## Schema (versión 1)

```jsonc
{
  "schemaVersion": 1,
  "id": "novela-tres-actos",
  "name": "Novela en tres actos",
  "description": "Estructura clásica con planteamiento, confrontación y resolución.",
  "kind": "novel", // "novel" | "paper" | "manga" | "screenplay" | "generic"
  "locale": "es", // "es" | "en"
  "tier": "free", // "free" | "premium" — gating del loader
  "structure": [
    {
      "title": "Acto 1 — Planteamiento",
      "docType": "folder", // ver DocumentType: chapter|scene|note|folder|manga_page
      "synopsis": "Mundo ordinario, incidente desencadenante, primer punto de giro.",
      "children": [{ "title": "Capítulo 1", "docType": "chapter" }],
    },
  ],
  "metadataFields": [
    {
      "key": "author",
      "label": "Autor/a",
      "type": "string", // "string" | "text" | "number" | "date"
      "required": true,
      "default": null,
    },
  ],
}
```

### Campos top-level

| Campo            | Tipo              | Requerido | Notas                                                      |
| ---------------- | ----------------- | --------- | ---------------------------------------------------------- |
| `schemaVersion`  | `u32`             | sí        | debe ser `1`                                               |
| `id`             | `string`          | sí        | identificador único, kebab-case (`novela-tres-actos`)      |
| `name`           | `string`          | sí        | nombre legible                                             |
| `description`    | `string`          | no        | descripción corta para wizard                              |
| `kind`           | `string`          | sí        | uno de: `novel \| paper \| manga \| screenplay \| generic` |
| `locale`         | `string`          | sí        | `es` o `en`                                                |
| `tier`           | `string`          | sí        | `free` o `premium`. Free MVP solo carga `free`             |
| `structure`      | `TemplateNode[]`  | sí        | árbol seed; puede estar vacío                              |
| `metadataFields` | `MetadataField[]` | no        | campos extra pedidos al usuario                            |

### `TemplateNode` (estructura recursiva)

| Campo      | Tipo             | Requerido | Notas                                                       |
| ---------- | ---------------- | --------- | ----------------------------------------------------------- |
| `title`    | `string`         | sí        | título del documento o folder                               |
| `docType`  | `DocumentType`   | sí        | uno de los tipos válidos en el dominio                      |
| `synopsis` | `string`         | no        | texto explicativo opcional, se guardará como contenido seed |
| `children` | `TemplateNode[]` | no        | hijos en orden                                              |

### `MetadataField`

| Campo      | Tipo     | Requerido | Notas                                                              |
| ---------- | -------- | --------- | ------------------------------------------------------------------ |
| `key`      | `string` | sí        | clave en el JSON `metadata` del proyecto (`author`, `genre`, etc.) |
| `label`    | `string` | sí        | etiqueta legible para el wizard                                    |
| `type`     | `string` | sí        | `string`, `text`, `number`, `date`                                 |
| `required` | `bool`   | no        | default `false`                                                    |
| `default`  | `JSON`   | no        | valor por defecto opcional                                         |

## Validaciones

El loader rechaza la plantilla si:

- `schemaVersion != 1`
- `id` o `name` vacíos
- `kind` no está en el set permitido
- `docType` de cualquier nodo no está en el dominio
- `metadataFields[*].type` no está en el set permitido

Errores del loader son `AppError::Invariant` con mensaje preciso (id de la plantilla + clave fallida).

## Comportamiento al crear proyecto

1. Usuario selecciona plantilla en el wizard.
2. Wizard pide los `metadataFields` (si los hay) y el título del proyecto.
3. Backend (`ProjectManager.create`) abre una **única transacción SQLite** y:
   - Inserta el `Project` con `metadata` JSON consolidado.
   - Recorre `structure` recursivamente y crea cada `Document` respetando orden y jerarquía.
4. Si cualquier paso falla, la transacción se descarta — el proyecto **no queda parcial**.

## Plantillas built-in del MVP

- `generic` — estructura libre (un solo capítulo placeholder)
- `novela-tres-actos` — 3 actos con capítulos seed
- `paper-imrad` — Abstract / Introduction / Methods / Results / Discussion / References
- `manga-shonen` — capítulos como folders con páginas hijas

Las plantillas premium (Save the Cat, Hero's Journey, Snowflake, Light Novel JP, etc.) **no se incluyen** en el MVP, solo el slot `tier: "premium"` reservado en el schema.

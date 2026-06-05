# Catálogo de voces (TTS) — manifest dinámico

Las voces Piper se sirven vía un manifest curado en R2:
`https://bins.draffity.com/voices/v1/manifest.json`. Los archivos `.onnx`/`.onnx.json`
viven en HuggingFace (`rhasspy/piper-voices`). La app lee el manifest, lo cachea en
`~/.draffity/cache/voice-manifest.json` y cae a una semilla built-in (2 voces) sin red.

## Regenerar y publicar

    node scripts/sync-voice-manifest.mjs > manifest.json
    aws s3 cp manifest.json s3://bins-draffity/voices/v1/manifest.json \
      --endpoint-url "$R2_ENDPOINT"

Requiere las credenciales R2 (`AWS_ACCESS_KEY_ID`/`AWS_SECRET_ACCESS_KEY`/`R2_ENDPOINT`,
las mismas de los binarios Whisper). Para destacar voces: editar `RECOMMENDED`/`FEATURED`
en el script y re-publicar. **No requiere release de la app.**

import { ref, type Ref } from 'vue';

/** Visibilidad módulo-compartida de la paleta ⌘K, para abrirla desde cualquier
 *  componente (atajo de teclado o botón) sin prop drilling. */
const visible = ref(false);

export function useCommandPalette(): {
  visible: Ref<boolean>;
  open: () => void;
  close: () => void;
  toggle: () => void;
} {
  return {
    visible,
    open: () => {
      visible.value = true;
    },
    close: () => {
      visible.value = false;
    },
    toggle: () => {
      visible.value = !visible.value;
    },
  };
}

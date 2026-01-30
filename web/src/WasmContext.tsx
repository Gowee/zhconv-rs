import React, {
  createContext,
  useContext,
  useEffect,
  useState,
  useRef,
} from "react";
import PACKAGE from "../package.json";

type WasmModule = {
  zhconv: (
    text: string,
    target: string,
    mediawiki: boolean,
    cgroup: string,
  ) => Promise<string>;
  get_build_timestamp: () => string;
  get_commit: () => string;
  get_mediawiki_commit: () => string;
  get_opencc_commit: () => string;
  infer_variant_confidence: (text: string) => string;
};

type WasmContextType = {
  wasm: WasmModule | null;
  useOpenCC: boolean;
  setUseOpenCC: (useOpenCC: boolean) => void;
};

const WasmContext = createContext<WasmContextType>({
  wasm: null,
  useOpenCC: false,
  setUseOpenCC: () => {},
});

export const useWasm = () => useContext(WasmContext);

export const WasmProvider: React.FC<React.PropsWithChildren<{}>> = ({
  children,
}) => {
  // wasmCache stores the loaded WebAssembly modules to prevent re-fetching and re-instantiating
  // them every time the OpenCC option is toggled. This improves performance and avoids
  // unnecessary loading indicator flashes.
  const wasmCache = useRef<{ [key: string]: WasmModule }>({});
  const [wasm, setWasm] = useState<WasmModule | null>(null);
  const [useOpenCC, setUseOpenCC] = useState(() => {
    const stored = localStorage.getItem(`${PACKAGE.name}-opencc`);
    return stored ? JSON.parse(stored) : true;
  });

  useEffect(() => {
    localStorage.setItem(`${PACKAGE.name}-opencc`, JSON.stringify(useOpenCC));
    const loadWasm = async () => {
      console.time("Wasm loading");
      const cacheKey = useOpenCC ? "opencc" : "default";
      // If the module is already in cache, use it directly.
      // This prevents setting wasm to null and avoids a loading indicator flash
      // if the user switches back and forth between already loaded modules.
      // Besides, dozens of ms delay is observed when relying on the automatic cache of browser.
      if (wasmCache.current[cacheKey]) {
        setWasm(wasmCache.current[cacheKey]);
        console.log(
          `Loaded cached zhconv ${useOpenCC ? "with" : "without"} OpenCC dicts.`,
        );
        console.timeEnd("Wasm loading");
        return;
      }

      // Set wasm to null to trigger the loading indicator in App.tsx
      setWasm(null);
      const wasmModule = useOpenCC
        ? await import("@pkg-opencc/zhconv.js")
        : await import("@pkg-default/zhconv.js");
      wasmCache.current[cacheKey] = wasmModule;
      setWasm(wasmModule);
      console.log(
        `Loaded zhconv ${useOpenCC ? "with" : "without"} OpenCC dicts.`,
      );
      console.timeEnd("Wasm loading");
    };
    loadWasm();
  }, [useOpenCC]);

  return (
    <WasmContext.Provider value={{ wasm, useOpenCC, setUseOpenCC }}>
      {children}
    </WasmContext.Provider>
  );
};

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

type AppContextType = {
  wasm: WasmModule | null;
  useOpenCC: boolean;
  setUseOpenCC: (useOpenCC: boolean) => void;
  cgroups: {
    data: { [name: string]: string };
    timestamp: number | null;
  };
};

const AppContext = createContext<AppContextType>({
  wasm: null,
  useOpenCC: false,
  setUseOpenCC: () => { },
  cgroups: { data: {}, timestamp: null },
});

export const useApp = () => useContext(AppContext);

// Alias for backward compatibility during refactor, or we can just remove it.
// keeping it temporarily might help if I miss one, but the goal is to replace.
// export const useWasm = useApp;

export const AppProvider: React.FC<React.PropsWithChildren> = ({
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

  const [cgroups, setCGroups] = useState<{
    data: { [name: string]: string };
    timestamp: number | null;
  }>({ data: {}, timestamp: null });

  // Load WASM
  useEffect(() => {
    localStorage.setItem(`${PACKAGE.name}-opencc`, JSON.stringify(useOpenCC));
    const loadWasm = async () => {
      const cacheKey = useOpenCC ? "opencc" : "default";
      const loadingLabel = `zhconv loading (${cacheKey})`
      console.time(loadingLabel);
      // If the module is already in cache, use it directly.
      // This prevents setting wasm to null and avoids a loading indicator flash
      // if the user switches back and forth between already loaded modules.
      // Besides, dozens of ms delay is observed when relying on the automatic cache of browser.
      if (wasmCache.current[cacheKey]) {
        setWasm(wasmCache.current[cacheKey]);
        console.log(
          `Using cached wasm`,
        );
        console.timeEnd(loadingLabel);
        return;
      }

      // Set wasm to null to trigger the loading indicator in App.tsx
      setWasm(null);
      const wasmModule = useOpenCC
        ? await import("@pkg-opencc/zhconv.js")
        : await import("@pkg-default/zhconv.js");
      wasmCache.current[cacheKey] = wasmModule;
      setWasm(wasmModule);
      console.timeEnd(loadingLabel);
    };
    loadWasm();
  }, [useOpenCC]);

  // Load cgroups.json
  useEffect(() => {
    async function loadCGroups() {
      try {
        const res = await fetch("/cgroups.json");
        const json = await res.json();
        setCGroups({
          data: json.data as { [name: string]: string },
          timestamp: json.timestamp as number,
        });
      } catch (e) {
        console.error("Failed to load cgroups.json", e);
      }
    }
    loadCGroups();
  }, []);

  return (
    <AppContext.Provider value={{ wasm, useOpenCC, setUseOpenCC, cgroups }}>
      {children}
    </AppContext.Provider>
  );
};

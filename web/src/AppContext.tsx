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
  } | null;
};

const AppContext = createContext<AppContextType>({
  wasm: null,
  useOpenCC: false,
  setUseOpenCC: () => { },
  cgroups: null,
});

export const useApp = () => useContext(AppContext);

// Alias for backward compatibility during refactor, or we can just remove it.
// keeping it temporarily might help if I miss one, but the goal is to replace.
// export const useWasm = useApp;

// Initialize useOpenCC from localStorage immediately
const getInitialUseOpenCC = () => {
  const stored = localStorage.getItem(`${PACKAGE.name}-opencc`);
  return stored ? JSON.parse(stored) : false;
};

const initialUseOpenCC = getInitialUseOpenCC();

// Start loading WASM immediately
const initialWasmPromise = initialUseOpenCC
  ? import("@pkg-opencc/zhconv.js")
  : import("@pkg-default/zhconv.js");

export const AppProvider: React.FC<React.PropsWithChildren> = ({
  children,
}) => {
  // wasmCache stores the loaded WebAssembly modules to prevent re-fetching and re-instantiating
  // them every time the OpenCC option is toggled. This improves performance and avoids
  // unnecessary loading indicator flashes.
  const wasmCache = useRef<{ [key: string]: WasmModule }>({});
  const [wasm, setWasm] = useState<WasmModule | null>(null);
  const [useOpenCC, setUseOpenCC] = useState(initialUseOpenCC);

  const [cgroups, setCGroups] = useState<{
    data: { [name: string]: string };
    timestamp: number | null;
  } | null>(null);

  // Load WASM
  useEffect(() => {
    let cancelled = false;
    localStorage.setItem(`${PACKAGE.name}-opencc`, JSON.stringify(useOpenCC));
    const loadWasm = async () => {
      const cacheKey = useOpenCC ? "opencc" : "default";
      const loadingLabel = `zhconv loading (${cacheKey})`;
      console.time(loadingLabel);

      // Check if this is the initial load and we have a matching preloaded promise
      if (useOpenCC === initialUseOpenCC && !wasmCache.current[cacheKey]) {
        try {
          // If we haven't cached it yet (which we haven't on first render),
          // reuse the in-flight promise started at module scope.
          // Note: If the user toggles *very* quickly before this resolves, 
          // we might need to be careful, but the effect dependency [useOpenCC] handles logic reuse.
          // Ideally: we just want to avoid starting a *new* import if the top-level one matches what we need.
          // We can put the result into cache once resolved.

          // Actually, simply waiting on the promise is fine. 
          // Whatever the promise resolves to is the module.
          const wasmModule = await initialWasmPromise;

          // Always cache the result, even if we are cancelled.
          wasmCache.current[cacheKey] = wasmModule;

          if (cancelled) return;

          // Re-check if the component still wants this specific mode (though effect cleanup/race conditions are rare here for initial load)
          // But effectively, we just want to populate cache with it.
          setWasm(wasmModule);
          console.log(`Using preloaded wasm`);
          console.timeEnd(loadingLabel);
          return;
        } catch (e) {
          console.error("Failed to load preloaded wasm", e);
          // Fallthrough to normal loading if something weird happened (unlikely)
        }
      }

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

      // Always cache, even if cancelled
      wasmCache.current[cacheKey] = wasmModule;

      if (cancelled) return;

      setWasm(wasmModule);
      console.timeEnd(loadingLabel);
    };
    loadWasm();
    return () => {
      cancelled = true;
    };
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

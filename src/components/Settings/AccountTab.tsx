import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Store } from "@tauri-apps/plugin-store";
import { Eye, EyeOff, Check, X, Loader2, AlertCircle, ChevronDown } from "lucide-react";
import { useOverlayStore, PROVIDERS, ModelWithMeta } from "@/store";
import { ProviderIcon } from "@/components/icons/ProviderIcon";

export function AccountTab() {
  const apiKeyStatus = useOverlayStore((s) => s.apiKeyStatus);
  const apiKeyLast4 = useOverlayStore((s) => s.apiKeyLast4);
  const setApiKeyStatus = useOverlayStore((s) => s.setApiKeyStatus);
  const setApiKeyLast4 = useOverlayStore((s) => s.setApiKeyLast4);
  const setModels = useOverlayStore((s) => s.setModels);
  const selectedProvider = useOverlayStore((s) => s.selectedProvider);
  const setSelectedProvider = useOverlayStore((s) => s.setSelectedProvider);

  const [inputValue, setInputValue] = useState("");
  const [baseUrlInput, setBaseUrlInput] = useState("");
  const [revealed, setRevealed] = useState(false);
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const [providerHasKey, setProviderHasKey] = useState<Record<string, boolean>>({});
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const providerRef = useRef(selectedProvider);

  const currentProvider = PROVIDERS.find(p => p.id === selectedProvider);
  const isLocal = currentProvider?.local ?? false;

  // Keep providerRef in sync
  useEffect(() => {
    providerRef.current = selectedProvider;
  }, [selectedProvider]);

  // On mount / provider change: check key (cloud) or run health check (local)
  useEffect(() => {
    const currentProv = selectedProvider;
    const provEntry = PROVIDERS.find(p => p.id === currentProv);
    const localProvider = provEntry?.local ?? false;

    if (localProvider) {
      // Local provider: load base URL from store, run health check
      const checkHealth = async () => {
        try {
          const store = await Store.load("settings.json");
          const storedUrl = await store.get<string>(
            currentProv === "ollama" ? "ollama_base_url" : "lmstudio_base_url"
          );
          if (providerRef.current !== currentProv) return;
          setBaseUrlInput(storedUrl ?? "");
          setApiKeyLast4("");
          setInputValue("");

          // Run health check via validate_api_key (backend performs GET health check)
          setApiKeyStatus("validating");
          try {
            await invoke("validate_api_key", { provider: currentProv, apiKey: "" });
            if (providerRef.current !== currentProv) return;
            setApiKeyStatus("valid");
            // Fetch models (empty for now, Phase 38 adds discovery)
            const models = await invoke<ModelWithMeta[]>(
              "fetch_models",
              { provider: currentProv, apiKey: "" }
            );
            if (providerRef.current !== currentProv) return;
            setModels(models);
          } catch {
            if (providerRef.current !== currentProv) return;
            setApiKeyStatus("invalid");
          }
        } catch {
          // Store access failed
        }
      };
      checkHealth();
    } else {
      // Cloud provider: existing checkStoredKey logic (unchanged)
      const checkStoredKey = async () => {
        try {
          const key = await invoke<string | null>("get_api_key", { provider: currentProv });
          if (providerRef.current !== currentProv) return;
          if (key) {
            setApiKeyLast4(key.slice(-4));
            setApiKeyStatus("validating");
            try {
              await invoke("validate_api_key", { provider: currentProv, apiKey: key });
              if (providerRef.current !== currentProv) return;
              const models = await invoke<ModelWithMeta[]>(
                "fetch_models",
                { provider: currentProv, apiKey: key }
              );
              if (providerRef.current !== currentProv) return;
              setApiKeyStatus("valid");
              setModels(models);
            } catch {
              if (providerRef.current !== currentProv) return;
              setApiKeyStatus("invalid");
            }
          }
        } catch {
          // No stored key
        }
      };
      checkStoredKey();
    }
  }, [selectedProvider]); // eslint-disable-line react-hooks/exhaustive-deps

  // Check which providers have stored keys / healthy servers when dropdown opens
  useEffect(() => {
    if (!dropdownOpen) return;
    const checkKeys = async () => {
      const result: Record<string, boolean> = {};
      for (const p of PROVIDERS) {
        if (p.local) {
          // Local: check health instead of keychain
          try {
            await invoke("validate_api_key", { provider: p.id, apiKey: "" });
            result[p.id] = true;
          } catch {
            result[p.id] = false;
          }
        } else {
          // Cloud: check keychain
          try {
            const key = await invoke<string | null>("get_api_key", { provider: p.id });
            result[p.id] = !!key;
          } catch {
            result[p.id] = false;
          }
        }
      }
      setProviderHasKey(result);
    };
    checkKeys();
  }, [dropdownOpen]);

  // Debounced validation when user types (cloud providers only)
  useEffect(() => {
    if (isLocal) return; // Local providers use URL debounce effect instead

    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }

    if (inputValue.length <= 10) {
      return;
    }

    const currentProvider = selectedProvider;
    debounceRef.current = setTimeout(async () => {
      setApiKeyStatus("validating");
      try {
        await invoke("validate_api_key", { provider: currentProvider, apiKey: inputValue });
        if (providerRef.current !== currentProvider) return;
        const models = await invoke<ModelWithMeta[]>(
          "fetch_models",
          { provider: currentProvider, apiKey: inputValue }
        );
        if (providerRef.current !== currentProvider) return;
        await invoke("save_api_key", { provider: currentProvider, key: inputValue });
        setApiKeyStatus("valid");
        setModels(models);
        setApiKeyLast4(inputValue.slice(-4));
      } catch (err) {
        if (providerRef.current !== currentProvider) return;
        const errStr = typeof err === "string" ? err : String(err);
        if (errStr.includes("invalid_key") || errStr.includes("invalid key")) {
          setApiKeyStatus("invalid");
        } else {
          setApiKeyStatus("error");
        }
      }
    }, 800);

    return () => {
      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
      }
    };
  }, [inputValue]); // eslint-disable-line react-hooks/exhaustive-deps

  // Debounced URL save + health check for local providers
  useEffect(() => {
    if (!isLocal) return;
    if (debounceRef.current) clearTimeout(debounceRef.current);

    debounceRef.current = setTimeout(async () => {
      const currentProv = selectedProvider;
      // Save to store (even empty -- empty means use default)
      try {
        const store = await Store.load("settings.json");
        const storeKey = currentProv === "ollama" ? "ollama_base_url" : "lmstudio_base_url";
        if (baseUrlInput.trim()) {
          await store.set(storeKey, baseUrlInput.trim());
        } else {
          await store.delete(storeKey);
        }
        await store.save();
      } catch {
        // Non-fatal
      }

      // Re-run health check
      setApiKeyStatus("validating");
      try {
        await invoke("validate_api_key", { provider: currentProv, apiKey: "" });
        if (providerRef.current !== currentProv) return;
        setApiKeyStatus("valid");
      } catch {
        if (providerRef.current !== currentProv) return;
        setApiKeyStatus("invalid");
      }
    }, 800);

    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
    };
  }, [baseUrlInput]); // eslint-disable-line react-hooks/exhaustive-deps

  const handleDelete = async () => {
    if (isLocal) {
      // Clear stored URL
      try {
        const store = await Store.load("settings.json");
        const storeKey = selectedProvider === "ollama" ? "ollama_base_url" : "lmstudio_base_url";
        await store.delete(storeKey);
        await store.save();
      } catch { /* Best-effort */ }
      setBaseUrlInput("");
      setApiKeyStatus("unknown");
      setModels([]);
    } else {
      // Existing cloud provider delete logic
      try {
        await invoke("delete_api_key", { provider: selectedProvider });
      } catch { /* Best-effort delete */ }
      setApiKeyStatus("unknown");
      setApiKeyLast4("");
      setModels([]);
      setInputValue("");
    }
  };

  const handleProviderSelect = async (providerId: string) => {
    if (providerId === selectedProvider) {
      setDropdownOpen(false);
      return;
    }
    setSelectedProvider(providerId);
    // Persist to settings.json
    try {
      const store = await Store.load("settings.json");
      await store.set("selectedProvider", providerId);
      await store.save();
    } catch {
      // Non-fatal
    }
    // Reset API key state for new provider
    setApiKeyStatus("unknown");
    setApiKeyLast4("");
    setInputValue("");
    setBaseUrlInput("");
    setModels([]);
    setDropdownOpen(false);
    // The useEffect on [selectedProvider] will re-trigger checkStoredKey automatically
  };

  const currentProviderName = currentProvider?.name ?? selectedProvider;

  const placeholder =
    inputValue.length === 0 && apiKeyLast4
      ? `****...${apiKeyLast4}`
      : `Paste your ${currentProviderName} API key`;

  return (
    <div className="flex flex-col gap-3">
      {/* Provider dropdown */}
      <div className="flex flex-col gap-1.5">
        <p className="text-white/40 text-xs uppercase tracking-wider">
          Provider
        </p>
        <div className="relative">
          <button
            type="button"
            onClick={() => setDropdownOpen((o) => !o)}
            className="w-full flex items-center gap-2 bg-white/8 border border-white/10 rounded-lg px-3 py-2 text-sm text-white cursor-default"
          >
            <div className="w-6 h-6 rounded-full bg-white/10 flex items-center justify-center shrink-0">
              <ProviderIcon provider={selectedProvider} size={12} className="text-white/70" />
            </div>
            <span className="flex-1 text-left">{currentProviderName}</span>
            <ChevronDown size={14} className="text-white/40" />
          </button>
          {dropdownOpen && (
            <div className="absolute top-full left-0 right-0 mt-1 z-50 bg-[#2a2a2c]/95 backdrop-blur-xl border border-white/10 rounded-lg overflow-y-auto max-h-60">
              {PROVIDERS.map((p) => (
                <button
                  key={p.id}
                  type="button"
                  onClick={() => handleProviderSelect(p.id)}
                  className={[
                    "w-full flex items-center gap-2 px-3 py-2 text-sm transition-colors cursor-default",
                    p.id === selectedProvider
                      ? "text-white bg-white/10"
                      : "text-white/70 hover:bg-white/8",
                  ].join(" ")}
                >
                  <div className="w-6 h-6 rounded-full bg-white/10 flex items-center justify-center shrink-0">
                    <ProviderIcon provider={p.id} size={12} className="text-white/70" />
                  </div>
                  <span className="flex-1 text-left">{p.name}</span>
                  {providerHasKey[p.id] && (
                    <Check size={14} className="text-green-400" />
                  )}
                </button>
              ))}
            </div>
          )}
        </div>
      </div>

      {isLocal ? (
        /* Local provider: Server URL input */
        <div className="flex flex-col gap-1.5">
          <p className="text-white/40 text-xs uppercase tracking-wider">
            Server URL
          </p>
          <div className="flex items-center gap-2">
            <div className="flex-1">
              <input
                type="text"
                value={baseUrlInput}
                onChange={(e) => setBaseUrlInput(e.target.value)}
                placeholder={selectedProvider === "ollama" ? "localhost:11434" : "localhost:1234"}
                className={[
                  "w-full bg-white/8 border border-white/10 rounded-lg",
                  "px-3 py-2 text-sm text-white placeholder-white/30",
                  "focus:outline-none focus:border-white/25 transition-colors",
                ].join(" ")}
                spellCheck={false}
                autoComplete="off"
              />
            </div>
            {/* Status indicator -- same as API key validation */}
            <div className="flex items-center min-w-[20px]">
              {apiKeyStatus === "validating" && (
                <Loader2 size={16} className="text-white/50 animate-spin" />
              )}
              {apiKeyStatus === "valid" && (
                <Check size={16} className="text-green-400" />
              )}
              {apiKeyStatus === "invalid" && (
                <X size={16} className="text-red-400" />
              )}
              {apiKeyStatus === "error" && (
                <AlertCircle size={16} className="text-red-400" />
              )}
            </div>
          </div>
          {/* Status message */}
          <div className="min-h-[16px]">
            {apiKeyStatus === "invalid" && (
              <p className="text-red-400/80 text-xs">Server not running</p>
            )}
          </div>
          {/* Reset URL button */}
          {baseUrlInput && (
            <button
              type="button"
              onClick={handleDelete}
              className="self-start text-xs text-white/30 hover:text-red-400/70 transition-colors cursor-default"
            >
              Reset to default
            </button>
          )}
        </div>
      ) : (
        /* Cloud provider: API Key section */
        <div className="flex flex-col gap-1.5">
          <p className="text-white/40 text-xs uppercase tracking-wider">
            API Key
          </p>

          {/* Input row */}
          <div className="flex items-center gap-2">
            <div className="flex-1 relative">
              <input
                type={revealed ? "text" : "password"}
                value={inputValue}
                onChange={(e) => setInputValue(e.target.value)}
                placeholder={placeholder}
                className={[
                  "w-full bg-white/8 border border-white/10 rounded-lg",
                  "px-3 py-2 text-sm text-white placeholder-white/30",
                  "focus:outline-none focus:border-white/25 transition-colors",
                  "pr-9",
                ].join(" ")}
                spellCheck={false}
                autoComplete="off"
              />
              <button
                type="button"
                onClick={() => setRevealed((r) => !r)}
                className="absolute right-2.5 top-1/2 -translate-y-1/2 text-white/30 hover:text-white/60 transition-colors cursor-default"
                aria-label={revealed ? "Hide API key" : "Show API key"}
              >
                {revealed ? <EyeOff size={14} /> : <Eye size={14} />}
              </button>
            </div>

            {/* Status indicator */}
            <div className="flex items-center min-w-[20px]">
              {apiKeyStatus === "validating" && (
                <Loader2 size={16} className="text-white/50 animate-spin" />
              )}
              {apiKeyStatus === "valid" && (
                <Check size={16} className="text-green-400" />
              )}
              {apiKeyStatus === "invalid" && (
                <X size={16} className="text-red-400" />
              )}
              {apiKeyStatus === "error" && (
                <AlertCircle size={16} className="text-red-400" />
              )}
            </div>
          </div>

          {/* Status message */}
          <div className="min-h-[16px]">
            {apiKeyStatus === "invalid" && (
              <p className="text-red-400/80 text-xs">Invalid API key</p>
            )}
            {apiKeyStatus === "error" && (
              <p className="text-red-400/80 text-xs">Validation failed</p>
            )}
          </div>

          {/* Delete key button */}
          {(apiKeyStatus === "valid" || apiKeyLast4) && (
            <button
              type="button"
              onClick={handleDelete}
              className="self-start text-xs text-white/30 hover:text-red-400/70 transition-colors cursor-default"
            >
              Remove key
            </button>
          )}
        </div>
      )}
    </div>
  );
}

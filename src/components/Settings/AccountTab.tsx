import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Eye, EyeOff, Check, X, Loader2, AlertCircle } from "lucide-react";
import { useOverlayStore, XaiModelWithMeta } from "@/store";

export function AccountTab() {
  const apiKeyStatus = useOverlayStore((s) => s.apiKeyStatus);
  const apiKeyLast4 = useOverlayStore((s) => s.apiKeyLast4);
  const setApiKeyStatus = useOverlayStore((s) => s.setApiKeyStatus);
  const setApiKeyLast4 = useOverlayStore((s) => s.setApiKeyLast4);
  const setModels = useOverlayStore((s) => s.setModels);

  const [inputValue, setInputValue] = useState("");
  const [revealed, setRevealed] = useState(false);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // On mount: check for existing stored key and validate it
  useEffect(() => {
    const checkStoredKey = async () => {
      try {
        const key = await invoke<string | null>("get_api_key");
        if (key) {
          setApiKeyLast4(key.slice(-4));
          setApiKeyStatus("validating");
          try {
            const models = await invoke<XaiModelWithMeta[]>(
              "validate_and_fetch_models",
              { apiKey: key }
            );
            setApiKeyStatus("valid");
            setModels(models);
          } catch {
            setApiKeyStatus("invalid");
          }
        }
      } catch {
        // No stored key -- leave status as "unknown"
      }
    };

    checkStoredKey();
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  // Debounced validation when user types
  useEffect(() => {
    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }

    if (inputValue.length <= 10) {
      return;
    }

    debounceRef.current = setTimeout(async () => {
      setApiKeyStatus("validating");
      try {
        const models = await invoke<XaiModelWithMeta[]>(
          "validate_and_fetch_models",
          { apiKey: inputValue }
        );
        await invoke("save_api_key", { key: inputValue });
        setApiKeyStatus("valid");
        setModels(models);
        setApiKeyLast4(inputValue.slice(-4));
      } catch (err) {
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

  const handleDelete = async () => {
    try {
      await invoke("delete_api_key");
    } catch {
      // Best-effort delete
    }
    setApiKeyStatus("unknown");
    setApiKeyLast4("");
    setModels([]);
    setInputValue("");
  };

  const placeholder =
    inputValue.length === 0 && apiKeyLast4
      ? `****...${apiKeyLast4}`
      : "Paste your xAI API key";

  return (
    <div className="flex flex-col gap-3">
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
          {apiKeyStatus === "valid" && apiKeyLast4 && (
            <p className="text-green-400/60 text-xs">
              Key ending in {apiKeyLast4} is valid
            </p>
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
    </div>
  );
}

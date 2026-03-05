// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  cleanup,
  fireEvent,
  render,
  screen
} from "@testing-library/react";

import { DictionaryWorkspace } from "./DictionaryWorkspace";

const mockSlice = vi.fn();

vi.mock("../../store/slices", () => ({
  useDictionaryWorkspaceSlice: () => mockSlice()
}));

const defaultSlice = {
  runtimeConfig: {
    dictionary: {
      enabled: true,
      search_mode: "prefix",
      max_results: 100,
      postgres: {
        host: "127.0.0.1",
        port: 5432,
        database: "data",
        schema: "dictionary"
      }
    }
  },
  dictionaryLanguages: ["en", "es"],
  dictionaryLanguage: "en",
  dictionaryQuery: "anchor",
  dictionaryLoading: false,
  dictionaryError: null,
  dictionaryResults: [
    {
      id: 1,
      word: "anchor",
      language: "en",
      part_of_speech: "noun",
      pronunciation: "/ˈæŋ.kɚ/",
      summary: "A heavy object used to secure a vessel.",
      source_table: "pages",
      matched_by_prefix: true
    },
    {
      id: 2,
      word: "anchored",
      language: "en",
      part_of_speech: "verb",
      pronunciation: null,
      summary: "Past tense of anchor.",
      source_table: "pages",
      matched_by_prefix: true
    }
  ],
  dictionaryTotal: 2,
  dictionaryTruncated: false,
  dictionaryWarnings: [],
  dictionaryEntry: {
    id: 1,
    word: "anchor",
    language: "en",
    part_of_speech: "noun",
    pronunciation: "/ˈæŋ.kɚ/",
    etymology: "From Latin ancora.",
    definitions: ["A heavy object used to secure a vessel."],
    senses: [{ order: 1, text: "A heavy object used to secure a vessel." }],
    pronunciations: [{ text: "/ˈæŋ.kɚ/", system: "ipa" }],
    examples: ["Drop anchor before the storm."],
    notes: ["Nautical usage."],
    metadata: [
      { relation_type: "synonym", target: "grapnel" },
      { relation_type: "antonym", target: "drift" }
    ],
    source_table: "pages/definitions/relations"
  },
  dictionarySelectedId: 1,
  loadDictionaryLanguages: vi.fn().mockResolvedValue(undefined),
  setDictionaryLanguage: vi.fn(),
  setDictionaryQuery: vi.fn(),
  searchDictionaryEntries: vi.fn().mockResolvedValue(undefined),
  selectDictionaryHit: vi.fn().mockResolvedValue(undefined)
};

describe("DictionaryWorkspace", () => {
  beforeEach(() => {
    cleanup();
    vi.clearAllMocks();
    mockSlice.mockReturnValue({ ...defaultSlice });
  });

  it("renders dictionary workspace sections with entry details", () => {
    render(<DictionaryWorkspace />);

    expect(screen.getByText("Dictionary")).toBeTruthy();
    expect(screen.getByText("backend: 127.0.0.1:5432/data.dictionary")).toBeTruthy();
    expect(screen.getByText("Definitions")).toBeTruthy();
    expect(screen.getByText("Pronunciation (IPA)")).toBeTruthy();
    expect(screen.getByText("Synonyms")).toBeTruthy();
    expect(screen.getByText("grapnel")).toBeTruthy();
  });

  it("supports keyboard navigation for result selection", () => {
    const selectDictionaryHit = vi.fn().mockResolvedValue(undefined);
    mockSlice.mockReturnValue({
      ...defaultSlice,
      selectDictionaryHit
    });

    render(<DictionaryWorkspace />);

    const input =
      screen.getAllByLabelText(
        "Search word"
      )[0];
    fireEvent.keyDown(input, { key: "ArrowDown" });

    expect(selectDictionaryHit).toHaveBeenCalledWith(
      expect.objectContaining({ id: 2, word: "anchored" })
    );

    fireEvent.keyDown(input, { key: "Enter" });
    expect(selectDictionaryHit).toHaveBeenCalled();
  });

  it("applies language changes without auto search", () => {
    const setDictionaryLanguage = vi.fn();
    const searchDictionaryEntries = vi.fn().mockResolvedValue(undefined);
    mockSlice.mockReturnValue({
      ...defaultSlice,
      setDictionaryLanguage,
      searchDictionaryEntries
    });

    render(<DictionaryWorkspace />);

    const language =
      screen.getByRole("combobox", {
        name: "Language"
      });
    fireEvent.mouseDown(language);
    fireEvent.click(screen.getByRole("option", { name: "es" }));

    expect(setDictionaryLanguage).toHaveBeenCalledWith("es");
    expect(searchDictionaryEntries).not.toHaveBeenCalled();
  });

  it("runs search only when Search button is clicked", () => {
    const setDictionaryQuery = vi.fn();
    const searchDictionaryEntries = vi.fn().mockResolvedValue(undefined);
    mockSlice.mockReturnValue({
      ...defaultSlice,
      dictionaryQuery: "",
      setDictionaryQuery,
      searchDictionaryEntries
    });

    render(<DictionaryWorkspace />);

    const input = screen.getAllByLabelText("Search word")[0];
    fireEvent.change(input, { target: { value: "rat" } });
    expect(searchDictionaryEntries).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole("button", { name: "Search" }));
    expect(setDictionaryQuery).toHaveBeenCalledWith("rat");
    expect(searchDictionaryEntries).toHaveBeenCalledTimes(1);
  });
});

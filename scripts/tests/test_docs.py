import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path

SPEC = importlib.util.spec_from_file_location("tiler_docs", Path(__file__).parents[1] / "docs.py")
docs = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = docs
SPEC.loader.exec_module(docs)


class FrontmatterTests(unittest.TestCase):
    def parse(self, frontmatter: str):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "README.md"
            path.write_text(frontmatter + "\n# Test\n", encoding="utf-8")
            return docs.parse(path, root)

    def test_accepts_strict_json_subset(self):
        record, errors = self.parse('---\nschema: "tiler-doc/v1"\nid: "tiler.portal.test"\nkind: "portal"\ntitle: "Test"\ntopics: ["test"]\n---')
        self.assertEqual(errors, [])
        self.assertEqual(record.meta["topics"], ["test"])

    def test_rejects_duplicate_and_nested_values(self):
        _, errors = self.parse('---\nid: "one"\nid: "two"\nextra: {"nested": true}\n---')
        self.assertTrue(any("duplicate field id" in error for error in errors))
        self.assertTrue(any("flat scalar array" in error for error in errors))

    def test_rejects_unknown_kind_field(self):
        record, errors = self.parse('---\nschema: "tiler-doc/v1"\nid: "tiler.portal.test"\nkind: "portal"\ntitle: "Test"\ntopics: ["test"]\nevidence: ["x"]\n---')
        self.assertEqual(errors, [])
        self.assertTrue(any("unknown field evidence" in error for error in docs.validate_record(record, Path("."))))


class GraphTests(unittest.TestCase):
    def test_duplicate_ids_and_unresolved_edges_fail(self):
        records = [
            docs.Record(Path("a.md"), {"id": "tiler.x", "kind": "portal", "related": ["tiler.missing"]}, ""),
            docs.Record(Path("b.md"), {"id": "tiler.x", "kind": "portal"}, ""),
        ]
        errors = docs.validate_graph(records, Path("."))
        self.assertTrue(any("duplicate id" in error for error in errors))
        self.assertTrue(any("unresolved related" in error for error in errors))

    def test_related_edge_has_one_canonical_direction(self):
        records = [
            docs.Record(Path("a.md"), {"id": "tiler.z", "kind": "portal", "related": ["tiler.a"]}, ""),
            docs.Record(Path("b.md"), {"id": "tiler.a", "kind": "portal"}, ""),
        ]
        errors = docs.validate_graph(records, Path("."))
        self.assertTrue(any("lexicographically smaller" in error for error in errors))

    def test_dependency_cycles_fail(self):
        records = [
            docs.Record(Path("a.md"), {"id": "tiler.a", "kind": "portal", "depends_on": ["tiler.b"]}, ""),
            docs.Record(Path("b.md"), {"id": "tiler.b", "kind": "portal", "depends_on": ["tiler.a"]}, ""),
        ]
        errors = docs.validate_graph(records, Path("."))
        self.assertTrue(any("contains a cycle" in error for error in errors))


if __name__ == "__main__":
    unittest.main()

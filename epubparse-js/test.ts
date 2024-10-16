import {readFileSync} from "fs"
import { epubToBook } from "./src/index.js"
import test from 'ava';

const nestedEpub = readFileSync('../test_resources/nested.epub');

test('epubToBook returns zip error for invalid bytes', t => {
	t.is(epubToBook(new Uint8Array([1, 2, 3])), 'Error in underlying Zip archive');
});

test('nestedEpub example is parsed correctly', t => {
	const parsed = epubToBook(nestedEpub);
	if (typeof parsed === "string") {
		t.fail()
	}
	else {
		t.is(parsed.title, "Nested example");
		t.is(parsed.author, "Jannes");
		t.is(parsed.chapters[0].title, "Nested example");
		t.is(parsed.chapters[1].title, "Chapter 1");
		t.is(parsed.chapters[2].title, "Chapter 2");
		t.is(parsed.chapters[3].title, "Chapter 3");
	}
});

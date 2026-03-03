# Search Engine

Search is powered by TF-IDF keyword extraction. The keyword extraction function is **shared** between frontend and backend — both use the same algorithm to produce identical keywords from the same content.

## Architecture

- **Frontend** computes keywords on every file save, updates a local search index stored in localStorage or IndexedDB. Search queries run entirely against this local index.
- **Backend** also computes keywords on file save, but only stores the `.kw.tgz` file alongside the document in S3. Backend does not maintain a search index.
- **Index rebuild**: when the frontend has no local index (new device, cleared storage), it requests the TOC from the backend, then fetches `.kw.tgz` files to reconstruct the index. This runs in a web worker so the UI stays responsive.

## TF-IDF Keyword Extraction

This function must produce identical output on both frontend and backend given the same input.

### Tokenization
1. Split text on whitespace and punctuation.
2. Lowercase all tokens.
3. Remove stopwords (common words like "the", "is", "at", "and", etc.).
4. Tokens shorter than 2 characters are discarded.

### Term Frequency (TF)
For each document, calculate the frequency of each token:
```
TF(term, doc) = (number of times term appears in doc) / (total number of terms in doc)
```

### Inverse Document Frequency (IDF)
IDF is calculated against the user's entire corpus (all their files):
```
IDF(term, user) = log(total documents by user / number of user documents containing term)
```

On the frontend, the corpus is the local index. On the backend, IDF is computed against the keywords already stored in S3 for that user. Keyword files of other documents are not re-generated on each save — IDF values in existing `.kw.tgz` files become stale and are acceptable.

### TF-IDF Score
```
TF-IDF(term, doc, user) = TF(term, doc) * IDF(term, user)
```

### Keyword Selection
- Compute TF-IDF for every token in the document.
- Sort by score descending.
- Keep the top N keywords (N = 30 by default).

## Storage Format

Keywords are stored as `/users/<user-id>/<path>.kw.tgz` — a compressed file containing one keyword per line, ordered by score descending:
```
keyword1
keyword2
keyword3
...
```

## Search (Frontend-Side)

Search runs entirely on the frontend against the local index:
1. Tokenize the query using the same tokenization rules.
2. For each file in the index, check its keyword list.
3. Score each file by the number of query tokens that appear in its keyword list, weighted by keyword position (earlier = higher rank).
4. Return files sorted by score descending.

## Index Rebuild Flow (Frontend)

When the frontend has no index:
1. Request TOC from backend (`GET /api/toc`), paginating until all items are fetched.
2. For each file path in the TOC, request its keyword file (`GET /api/files/keywords?path=<path>`).
3. Build the search index from the downloaded keywords.
4. Store the index in localStorage or IndexedDB.
5. All of this runs in a web worker to keep the UI responsive.

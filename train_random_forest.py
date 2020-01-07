import sklearn
import numpy as np
import json
from sklearn.feature_extraction import DictVectorizer
from sklearn.model_selection import KFold
from sklearn.ensemble import ExtraTreesClassifier
from sklearn.metrics import roc_auc_score, precision_score, recall_score

# This may not be very stable across sklearn versions...
from sklearn.tree import _tree
from collections import defaultdict

# If it doesn't have the POETRY label, it's not.
# Need numeric labels for learning.
def label_to_y(label):
    if label == "POETRY":
        return 1
    else:
        return 0

# Recursively flatten nested lists
def flatten(xs):
    if type(xs) == list:
        for x in xs:
            for y in flatten(x):
                yield y
    else:
        yield xs

# recursively flatten, sort, and convert to a numpy array.
# Used later to get the pages for each book into a big list of pages.
def flat_arr(xs):
    return np.array(sorted(flatten(xs)))

# Load the truth data from the JSONL file into these parallel lists:
data = []
ys = []
books = []
pages = []

with open("truth-data/truth.jsonl") as fp:
    for line in fp:
        instance = json.loads(line)
        data.append(instance["features"])
        ys.append(label_to_y(instance["label"]))
        books.append(instance["book"])
        pages.append(instance["page"])

# assign numbers to teach feature
fnums = DictVectorizer()
xs = fnums.fit_transform(data)
fnames = fnums.get_feature_names()

# Get the truth data into sliceable numpy array
ys = np.array(ys)

# Group pages by book:
by_book = defaultdict(list)
for i in range(len(data)):
    by_book[books[i]].append(i)

# construct a list of books:
by_book = dict((book, nums) for book, nums in by_book.items())
books = np.array(sorted(set(books)))

# collect models from each of the folds:
models = []
measures = defaultdict(list)

# split by book for robust training:
folds = KFold(n_splits=10, shuffle=True, random_state=42)
for train_b, test_b in folds.split(books):
    train_books = books[train_b]
    test_books = books[test_b]

    # get the page ids for each book:
    train_i = flat_arr([by_book[b] for b in train_books])
    test_i = flat_arr([by_book[b] for b in test_books])

    # train model on pages
    model = ExtraTreesClassifier(
        n_estimators=30, random_state=13, class_weight="balanced"
    )
    model.fit(xs[train_i], ys[train_i])
    # hold onto it
    models.append(model)

    # evaluate as we go
    yp = model.predict_proba(xs[test_i])[:, 1]
    AUC = roc_auc_score(ys[test_i], yp)
    measures["AUC"].append(AUC)
    print("AUC: %1.3f" % AUC)


def dump_tree(tree_model):
    """Recursively turn a SKLearn Tree model into a python dictionary (which can be saved as JSON)"""
    tree = tree_model.tree_

    def recurse(node):
        """Recursively handle a given node."""
        if tree.feature[node] != _tree.TREE_UNDEFINED:
            fid = int(tree.feature[node])
            threshold = float(tree.threshold[node])
            return {
                "fid": fid,
                "threshold": threshold,
                "lhs": recurse(tree.children_left[node]),
                "rhs": recurse(tree.children_right[node]),
            }
        else:
            return {"leaf": tree.value[node][0].tolist()}

    return recurse(0)

# Saving the feature names allows us to tell if this model is out of date
forest = {
    "feature_names": fnames,
    "forest": [[dump_tree(e) for e in m.estimators_] for m in models],
}

# Actually save the data here:
with open("forest.json", "w") as fp:
    json.dump(forest, fp)


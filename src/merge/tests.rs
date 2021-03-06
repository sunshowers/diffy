use super::*;

macro_rules! assert_merge {
    ($original:ident, $ours:ident, $theirs:ident, $kind:ident($expected:expr), $msg:literal $(,)?) => {
        let solution = merge($original, $ours, $theirs);

        macro_rules! result {
            (Ok, $s:expr) => {
                Result::<&str, &str>::Ok($s)
            };
            (Err, $s:expr) => {
                Result::<&str, &str>::Err($s)
            };
        }
        assert!(
            same_merge(result!($kind, $expected), &solution),
            concat!($msg, "\nexpected={:#?}\nactual={:#?}"),
            result!($kind, $expected),
            solution
        );

        let solution_bytes =
            merge_bytes($original.as_bytes(), $ours.as_bytes(), $theirs.as_bytes());

        macro_rules! result_bytes {
            (Ok, $s:expr) => {
                Result::<&[u8], &[u8]>::Ok($s.as_bytes())
            };
            (Err, $s:expr) => {
                Result::<&[u8], &[u8]>::Err($s.as_bytes())
            };
        }
        assert!(
            same_merge_bytes(result_bytes!($kind, $expected), &solution_bytes),
            concat!($msg, "\nexpected={:#?}\nactual={:#?}"),
            result_bytes!($kind, $expected),
            solution_bytes
        );
    };
}

fn same_merge(expected: Result<&str, &str>, actual: &Result<String, String>) -> bool {
    match (expected, actual) {
        (Ok(expected), Ok(actual)) => expected == actual,
        (Err(expected), Err(actual)) => expected == actual,
        (_, _) => false,
    }
}

fn same_merge_bytes(expected: Result<&[u8], &[u8]>, actual: &Result<Vec<u8>, Vec<u8>>) -> bool {
    match (expected, actual) {
        (Ok(expected), Ok(actual)) => expected == &actual[..],
        (Err(expected), Err(actual)) => expected == &actual[..],
        (_, _) => false,
    }
}

#[test]
fn test_merge() {
    let original = "\
carrots
garlic
onions
salmon
mushrooms
tomatoes
salt
";
    let a = "\
carrots
salmon
mushrooms
tomatoes
garlic
onions
salt
";
    let b = "\
carrots
salmon
garlic
onions
mushrooms
tomatoes
salt
";

    assert_merge!(original, original, original, Ok(original), "Equal case #1");
    assert_merge!(original, a, a, Ok(a), "Equal case #2");
    assert_merge!(original, b, b, Ok(b), "Equal case #3");

    let expected = "\
carrots
<<<<<<< ours
salmon
||||||| original
garlic
onions
salmon
=======
salmon
garlic
onions
>>>>>>> theirs
mushrooms
tomatoes
garlic
onions
salt
";

    assert_merge!(original, a, b, Err(expected), "Single Conflict case");

    let expected = "\
carrots
<<<<<<< ours
salmon
garlic
onions
||||||| original
garlic
onions
salmon
=======
salmon
>>>>>>> theirs
mushrooms
tomatoes
garlic
onions
salt
";

    assert_merge!(
        original,
        b,
        a,
        Err(expected),
        "Reverse Single Conflict case"
    );

    let original = "\
carrots
garlic
onions
salmon
tomatoes
salt
";
    let a = "\
carrots
salmon
tomatoes
garlic
onions
salt
";
    let b = "\
carrots
salmon
garlic
onions
tomatoes
salt
";
    let expected = "\
carrots
<<<<<<< ours
salmon
tomatoes
||||||| original
=======
salmon
>>>>>>> theirs
garlic
onions
<<<<<<< ours
||||||| original
salmon
tomatoes
=======
tomatoes
>>>>>>> theirs
salt
";

    assert_merge!(original, a, b, Err(expected), "Multiple Conflict case");

    let expected = "\
carrots
<<<<<<< ours
salmon
||||||| original
=======
salmon
tomatoes
>>>>>>> theirs
garlic
onions
<<<<<<< ours
tomatoes
||||||| original
salmon
tomatoes
=======
>>>>>>> theirs
salt
";
    assert_merge!(
        original,
        b,
        a,
        Err(expected),
        "Reverse Multiple Conflict case"
    );
}

#[test]
fn myers_diffy_vs_git() {
    let original = "\
void Chunk_copy(Chunk *src, size_t src_start, Chunk *dst, size_t dst_start, size_t n)
{
    if (!Chunk_bounds_check(src, src_start, n)) return;
    if (!Chunk_bounds_check(dst, dst_start, n)) return;

    memcpy(dst->data + dst_start, src->data + src_start, n);
}

int Chunk_bounds_check(Chunk *chunk, size_t start, size_t n)
{
    if (chunk == NULL) return 0;

    return start <= chunk->length && n <= chunk->length - start;
}
";
    let a = "\
int Chunk_bounds_check(Chunk *chunk, size_t start, size_t n)
{
    if (chunk == NULL) return 0;

    return start <= chunk->length && n <= chunk->length - start;
}

void Chunk_copy(Chunk *src, size_t src_start, Chunk *dst, size_t dst_start, size_t n)
{
    if (!Chunk_bounds_check(src, src_start, n)) return;
    if (!Chunk_bounds_check(dst, dst_start, n)) return;

    memcpy(dst->data + dst_start, src->data + src_start, n);
}
";
    let b = "\
void Chunk_copy(Chunk *src, size_t src_start, Chunk *dst, size_t dst_start, size_t n)
{
    if (!Chunk_bounds_check(src, src_start, n)) return;
    if (!Chunk_bounds_check(dst, dst_start, n)) return;

    // copy the bytes
    memcpy(dst->data + dst_start, src->data + src_start, n);
}

int Chunk_bounds_check(Chunk *chunk, size_t start, size_t n)
{
    if (chunk == NULL) return 0;

    return start <= chunk->length && n <= chunk->length - start;
}
";

    // TODO investigate why this doesn't match git's output
    let _expected_git = "\
int Chunk_bounds_check(Chunk *chunk, size_t start, size_t n)
{
    if (chunk == NULL) return 0;

<<<<<<< ours
    return start <= chunk->length && n <= chunk->length - start;
||||||| original
    memcpy(dst->data + dst_start, src->data + src_start, n);
=======
    // copy the bytes
    memcpy(dst->data + dst_start, src->data + src_start, n);
>>>>>>> theirs
}

void Chunk_copy(Chunk *src, size_t src_start, Chunk *dst, size_t dst_start, size_t n)
{
    if (!Chunk_bounds_check(src, src_start, n)) return;
    if (!Chunk_bounds_check(dst, dst_start, n)) return;

    memcpy(dst->data + dst_start, src->data + src_start, n);
}
";

    let expected_diffy = "\
int Chunk_bounds_check(Chunk *chunk, size_t start, size_t n)
{
    if (chunk == NULL) return 0;

    return start <= chunk->length && n <= chunk->length - start;
}

void Chunk_copy(Chunk *src, size_t src_start, Chunk *dst, size_t dst_start, size_t n)
{
    if (!Chunk_bounds_check(src, src_start, n)) return;
    if (!Chunk_bounds_check(dst, dst_start, n)) return;

    // copy the bytes
    memcpy(dst->data + dst_start, src->data + src_start, n);
}
";

    assert_merge!(original, a, b, Ok(expected_diffy), "Myers diffy merge");
}

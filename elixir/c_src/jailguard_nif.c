#include "erl_nif.h"
#include "jailguard.h"

#include <stdint.h>
#include <string.h>

static ERL_NIF_TERM atom(ErlNifEnv *env, const char *name) {
    return enif_make_atom(env, name);
}

static ERL_NIF_TERM ok(ErlNifEnv *env, ERL_NIF_TERM value) {
    return enif_make_tuple2(env, atom(env, "ok"), value);
}

static ERL_NIF_TERM error(ErlNifEnv *env, const char *reason) {
    return enif_make_tuple2(env, atom(env, "error"), atom(env, reason));
}

static const char *error_reason(int code) {
    switch (code) {
    case JAILGUARD_INVALID_INPUT:
        return "invalid_input";
    case JAILGUARD_DOWNLOAD_FAILED:
        return "download_failed";
    case JAILGUARD_INFERENCE_FAILED:
        return "inference_failed";
    case JAILGUARD_INTERNAL_ERROR:
        return "internal";
    default:
        return "unknown";
    }
}

static ERL_NIF_TERM error_from_code(ErlNifEnv *env, int code) {
    return error(env, error_reason(code));
}

static ERL_NIF_TERM risk_atom(ErlNifEnv *env, jailguard_risk_t risk) {
    switch (risk) {
    case Safe:
        return atom(env, "safe");
    case Low:
        return atom(env, "low");
    case Medium:
        return atom(env, "medium");
    case High:
        return atom(env, "high");
    case Critical:
        return atom(env, "critical");
    default:
        return atom(env, "unknown");
    }
}

static ERL_NIF_TERM result_to_term(ErlNifEnv *env, jailguard_detection_result_t result) {
    return enif_make_tuple4(
        env,
        result.is_injection ? atom(env, "true") : atom(env, "false"),
        enif_make_double(env, (double)result.score),
        enif_make_double(env, (double)result.confidence),
        risk_atom(env, result.risk));
}

static int copy_binary_to_cstring(ErlNifBinary bin, char **out) {
    if (memchr(bin.data, '\0', bin.size) != NULL) {
        return 0;
    }

    char *buffer = enif_alloc(bin.size + 1);
    if (buffer == NULL) {
        return -1;
    }

    memcpy(buffer, bin.data, bin.size);
    buffer[bin.size] = '\0';
    *out = buffer;
    return 1;
}

static ERL_NIF_TERM version_nif(ErlNifEnv *env, int argc, const ERL_NIF_TERM argv[]) {
    (void)argc;
    (void)argv;
    const char *version = jailguard_version();
    size_t len = strlen(version);
    ERL_NIF_TERM term;
    unsigned char *bytes = enif_make_new_binary(env, len, &term);
    memcpy(bytes, version, len);
    return term;
}

static ERL_NIF_TERM download_model_nif(ErlNifEnv *env, int argc, const ERL_NIF_TERM argv[]) {
    (void)argc;
    (void)argv;
    int rc = jailguard_download_model();
    if (rc != JAILGUARD_OK) {
        return error_from_code(env, rc);
    }
    return atom(env, "ok");
}

static ERL_NIF_TERM model_cache_dir_nif(ErlNifEnv *env, int argc, const ERL_NIF_TERM argv[]) {
    (void)argc;
    (void)argv;
    char *path = jailguard_model_cache_dir();
    if (path == NULL) {
        return error(env, "internal");
    }

    size_t len = strlen(path);
    ERL_NIF_TERM term;
    unsigned char *bytes = enif_make_new_binary(env, len, &term);
    memcpy(bytes, path, len);
    jailguard_free_string(path);
    return ok(env, term);
}

static ERL_NIF_TERM detect_nif(ErlNifEnv *env, int argc, const ERL_NIF_TERM argv[]) {
    if (argc != 1) {
        return enif_make_badarg(env);
    }

    ErlNifBinary bin;
    if (!enif_inspect_binary(env, argv[0], &bin)) {
        return error(env, "invalid_input");
    }

    char *text = NULL;
    int copied = copy_binary_to_cstring(bin, &text);
    if (copied == 0) {
        return error(env, "invalid_input");
    }
    if (copied < 0) {
        return error(env, "internal");
    }

    jailguard_detection_result_t result;
    int rc = jailguard_detect(text, &result);
    enif_free(text);

    if (rc != JAILGUARD_OK) {
        return error_from_code(env, rc);
    }

    return ok(env, result_to_term(env, result));
}

static ERL_NIF_TERM is_injection_nif(ErlNifEnv *env, int argc, const ERL_NIF_TERM argv[]) {
    if (argc != 1) {
        return enif_make_badarg(env);
    }

    ErlNifBinary bin;
    if (!enif_inspect_binary(env, argv[0], &bin)) {
        return error(env, "invalid_input");
    }

    char *text = NULL;
    int copied = copy_binary_to_cstring(bin, &text);
    if (copied == 0) {
        return error(env, "invalid_input");
    }
    if (copied < 0) {
        return error(env, "internal");
    }

    int out = 0;
    int rc = jailguard_is_injection(text, &out);
    enif_free(text);

    if (rc != JAILGUARD_OK) {
        return error_from_code(env, rc);
    }

    return ok(env, out ? atom(env, "true") : atom(env, "false"));
}

static ERL_NIF_TERM score_nif(ErlNifEnv *env, int argc, const ERL_NIF_TERM argv[]) {
    if (argc != 1) {
        return enif_make_badarg(env);
    }

    ErlNifBinary bin;
    if (!enif_inspect_binary(env, argv[0], &bin)) {
        return error(env, "invalid_input");
    }

    char *text = NULL;
    int copied = copy_binary_to_cstring(bin, &text);
    if (copied == 0) {
        return error(env, "invalid_input");
    }
    if (copied < 0) {
        return error(env, "internal");
    }

    float out = 0.0f;
    int rc = jailguard_score(text, &out);
    enif_free(text);

    if (rc != JAILGUARD_OK) {
        return error_from_code(env, rc);
    }

    return ok(env, enif_make_double(env, (double)out));
}

static void free_texts(char **texts, unsigned int count) {
    if (texts == NULL) {
        return;
    }
    for (unsigned int i = 0; i < count; i++) {
        if (texts[i] != NULL) {
            enif_free(texts[i]);
        }
    }
    enif_free(texts);
}

static ERL_NIF_TERM detect_batch_nif(ErlNifEnv *env, int argc, const ERL_NIF_TERM argv[]) {
    if (argc != 1) {
        return enif_make_badarg(env);
    }

    unsigned int count = 0;
    if (!enif_get_list_length(env, argv[0], &count)) {
        return error(env, "invalid_input");
    }
    if (count == 0) {
        return ok(env, enif_make_list(env, 0));
    }

    char **texts = enif_alloc(sizeof(char *) * count);
    jailguard_detection_result_t *results =
        enif_alloc(sizeof(jailguard_detection_result_t) * count);
    if (texts == NULL || results == NULL) {
        if (texts != NULL) {
            enif_free(texts);
        }
        if (results != NULL) {
            enif_free(results);
        }
        return error(env, "internal");
    }
    memset(texts, 0, sizeof(char *) * count);

    ERL_NIF_TERM list = argv[0];
    ERL_NIF_TERM head;
    ERL_NIF_TERM tail;
    for (unsigned int i = 0; i < count; i++) {
        ErlNifBinary bin;
        if (!enif_get_list_cell(env, list, &head, &tail) ||
            !enif_inspect_binary(env, head, &bin)) {
            free_texts(texts, count);
            enif_free(results);
            return error(env, "invalid_input");
        }

        int copied = copy_binary_to_cstring(bin, &texts[i]);
        if (copied == 0) {
            free_texts(texts, count);
            enif_free(results);
            return error(env, "invalid_input");
        }
        if (copied < 0) {
            free_texts(texts, count);
            enif_free(results);
            return error(env, "internal");
        }
        list = tail;
    }

    int rc = jailguard_detect_batch((const char *const *)texts, (uintptr_t)count, results);
    if (rc != JAILGUARD_OK) {
        free_texts(texts, count);
        enif_free(results);
        return error_from_code(env, rc);
    }

    ERL_NIF_TERM out = enif_make_list(env, 0);
    for (unsigned int i = count; i > 0; i--) {
        ERL_NIF_TERM item = result_to_term(env, results[i - 1]);
        out = enif_make_list_cell(env, item, out);
    }

    free_texts(texts, count);
    enif_free(results);
    return ok(env, out);
}

static ErlNifFunc nif_funcs[] = {
    {"version", 0, version_nif, 0},
    {"download_model", 0, download_model_nif, ERL_NIF_DIRTY_JOB_IO_BOUND},
    {"model_cache_dir", 0, model_cache_dir_nif, 0},
    {"detect", 1, detect_nif, ERL_NIF_DIRTY_JOB_CPU_BOUND},
    {"is_injection", 1, is_injection_nif, ERL_NIF_DIRTY_JOB_CPU_BOUND},
    {"score", 1, score_nif, ERL_NIF_DIRTY_JOB_CPU_BOUND},
    {"detect_batch", 1, detect_batch_nif, ERL_NIF_DIRTY_JOB_CPU_BOUND},
};

ERL_NIF_INIT(Elixir.JailGuard.Native, nif_funcs, NULL, NULL, NULL, NULL)

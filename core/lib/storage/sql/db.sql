DROP TABLE IF EXISTS t_block_witness_cloud;
CREATE TABLE t_block_witness_cloud
(
    f_id             bigserial PRIMARY KEY,
    f_block          BIGINT NOT NULL,
    f_version        BIGINT NOT NULL,
    f_object_key     text   NOT NULL,
    f_object_witness text   NOT NULL
);

DROP TABLE IF EXISTS t_prover_job_queue_cloud;
CREATE TABLE t_prover_job_queue_cloud
(
    f_id           bigserial PRIMARY KEY,
    f_job_status   INTEGER                  NOT NULL,
    f_job_priority INTEGER                  NOT NULL,
    f_job_type     TEXT                     NOT NULL,
    f_created_at   timestamp with time zone NOT NULL DEFAULT now(),
    f_version      BIGINT                   NOT NULL,
    f_updated_by   TEXT                     NOT NULL,
    f_updated_at   timestamp with time zone NOT NULL,
    f_first_block  BIGINT                   NOT NULL,
    f_last_block   BIGINT                   NOT NULL,
    f_object_key   TEXT                     NOT NULL,
    f_object_job   TEXT                     NOT NULL
);

DROP TABLE IF EXISTS t_proofs;
CREATE TABLE t_proofs
(
    f_id           bigserial PRIMARY KEY,
    f_block_number BIGINT                   NOT NULL,
    f_proof        jsonb                    NOT NULL,
    f_created_at   TIMESTAMP with time zone NOT NULL DEFAULT now()
);
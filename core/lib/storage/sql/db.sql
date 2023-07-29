DROP TABLE t_block_witness_cloud;
CREATE TABLE t_block_witness_cloud
(
    f_id             bigserial PRIMARY KEY,
    f_block          BIGINT NOT NULL,
    f_version        BIGINT NOT NULL,
    f_object_key     text   NOT NULL,
    f_object_witness text   NOT NULL
);
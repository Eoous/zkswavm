use halo2_proofs::{
    arithmetic::FieldExt,
    plonk::{Advice, Column, ConstraintSystem, Expression, VirtualCells},
};
use num_bigint::BigUint;
use specs::itable::OpcodeClass;
use std::marker::PhantomData;

use crate::circuits::event::EventCommonConfig;
use crate::circuits::event::{EventOpcodeConfig, EventOpcodeConfigBuilder};
use crate::circuits::instruction::InstructionConfig;
use crate::circuits::jump::JumpConfig;
use crate::circuits::memory::MemoryConfig;
use crate::{constant, constant_from, cur, utils::bn_to_field};

pub struct ConstConfig<F: FieldExt> {
    vtype: Column<Advice>,
    value: Column<Advice>,
    enable: Column<Advice>,
    _mark: PhantomData<F>,
}

pub struct ConstConfigBuilder {}

impl<F: FieldExt> EventOpcodeConfigBuilder<F> for ConstConfigBuilder {
    fn configure(
        meta: &mut ConstraintSystem<F>,
        common: &EventCommonConfig,
        opcode_bit: Column<Advice>,
        cols: &mut impl Iterator<Item = Column<Advice>>,
        instruction_table: &InstructionConfig<F>,
        memory_table: &MemoryConfig<F>,
        jump_table: &JumpConfig<F>,
    ) -> Box<dyn EventOpcodeConfig<F>> {
        let value = cols.next().unwrap();
        let vtype = cols.next().unwrap();

        memory_table.configure_stack_write_in_table(
            "const mlookup",
            "const mlookup rev",
            meta,
            |meta| cur!(meta, opcode_bit),
            |meta| cur!(meta, common.eid),
            |meta| constant_from!(1u64),
            |meta| cur!(meta, common.sp),
            |meta| cur!(meta, vtype),
            |meta| cur!(meta, value),
        );

        Box::new(ConstConfig {
            enable: opcode_bit,
            value,
            vtype,
            _mark: PhantomData,
        })
    }
}

impl<F: FieldExt> EventOpcodeConfig<F> for ConstConfig<F> {
    fn opcode(&self, meta: &mut VirtualCells<'_, F>) -> Expression<F> {
        // [(2 + vartype) << (64+13) + value] * enable
        // 2(10) << 77
        (constant!(bn_to_field(&(BigUint::from(OpcodeClass::Const as u64) << (64 + 13))))
            // vartype * (1 << 77)
            + cur!(meta, self.vtype) * constant!(bn_to_field(&(BigUint::from(1u64) << (64 + 13))))
            // value
            + cur!(meta, self.value))
            * cur!(meta, self.enable)
    }

    fn sp_diff(&self, meta: &mut VirtualCells<'_, F>) -> Expression<F> {
        // 1 * enable
        // 0 || 1
        constant_from!(1u64) * cur!(meta, self.enable)
    }
}

#[cfg(test)]
mod tests {
    use crate::runtime::{WasmInterpreter, WasmRuntime};
    use halo2_proofs::pairing::bn256::Fr as Fp;
    use wasmi::{ImportsBuilder, ModuleInstance};

    use crate::test::test_circuit_builder::run_test_circuit;

    #[test]
    fn test_ok() {
        let textual_repr = r#"
                (module
                    (func (export "test")
                      (i32.const 0)
                      (drop)
                    )
                   )
                "#;

        let compiler = WasmInterpreter::new();
        let compiled_module = compiler.compile(textual_repr).unwrap();
        let execution_log = compiler.run(&compiled_module, "test", vec![]).unwrap();

        run_test_circuit::<Fp>(compiled_module.tables, execution_log.tables).unwrap()
    }
}

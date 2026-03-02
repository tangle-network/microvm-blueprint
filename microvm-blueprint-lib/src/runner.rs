use crate::{
    errors::BlueprintResult,
    jobs::LifecycleJob,
    provider::VmProvider,
};

#[derive(Debug, Clone)]
pub struct JobRunner<P> {
    provider: P,
}

impl<P> JobRunner<P>
where
    P: VmProvider,
{
    pub fn new(provider: P) -> Self {
        Self { provider }
    }

    pub fn execute(&self, job: LifecycleJob) -> BlueprintResult<()> {
        match job {
            LifecycleJob::Create { vm_id } => self.provider.create_vm(&vm_id),
            LifecycleJob::Start { vm_id } => self.provider.start_vm(&vm_id),
            LifecycleJob::Stop { vm_id } => self.provider.stop_vm(&vm_id),
            LifecycleJob::Snapshot { vm_id, snapshot_id } => {
                self.provider.snapshot_vm(&vm_id, &snapshot_id)
            }
            LifecycleJob::Destroy { vm_id } => self.provider.destroy_vm(&vm_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        jobs::LifecycleJob,
        model::VmStatus,
        provider::{MockVmProvider, VmQuery},
        runner::JobRunner,
    };

    #[test]
    fn executes_lifecycle_jobs() {
        let provider = MockVmProvider::default();
        let runner = JobRunner::new(provider.clone());

        runner
            .execute(LifecycleJob::Create {
                vm_id: "vm-a".to_owned(),
            })
            .expect("create should succeed");
        runner
            .execute(LifecycleJob::Start {
                vm_id: "vm-a".to_owned(),
            })
            .expect("start should succeed");
        runner
            .execute(LifecycleJob::Snapshot {
                vm_id: "vm-a".to_owned(),
                snapshot_id: "snap-1".to_owned(),
            })
            .expect("snapshot should succeed");
        runner
            .execute(LifecycleJob::Stop {
                vm_id: "vm-a".to_owned(),
            })
            .expect("stop should succeed");
        runner
            .execute(LifecycleJob::Destroy {
                vm_id: "vm-a".to_owned(),
            })
            .expect("destroy should succeed");

        let vm = provider
            .get_vm("vm-a")
            .expect("query should succeed")
            .expect("vm should exist");

        assert_eq!(vm.status, VmStatus::Destroyed);
        assert_eq!(vm.snapshots, vec!["snap-1".to_owned()]);
    }
}

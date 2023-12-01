import Modal from '@/components/elements/Modal'

interface Props {
    open: boolean
}

const UnsupportedConsoleModal = ({ open }: Props) => {
    return (
        <Modal open={open} onClose={() => {}}>
            <Modal.Header>
                <Modal.Title>Unsupported Console</Modal.Title>
            </Modal.Header>

            <Modal.Body>
                <Modal.Description>
                    The console you are attempting to access is not supported by Convoy. Please return back to Convoy and relaunch your web console.
                </Modal.Description>
            </Modal.Body>
            <Modal.Actions>
                <Modal.Action>OK</Modal.Action>
            </Modal.Actions>
        </Modal>
    )
}

export default UnsupportedConsoleModal
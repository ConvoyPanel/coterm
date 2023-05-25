import Modal from '@/components/elements/Modal'

interface Props {
    open: boolean
}

const NoTokenFoundModal = ({ open }: Props) => {
    return (
        <Modal open={open} onClose={() => {}}>
            <Modal.Header>
                <Modal.Title>No Token Found</Modal.Title>
            </Modal.Header>

            <Modal.Body>
                <Modal.Description>
                    No token was found in your browser. Please log out and log
                    back in to resolve this issue.
                </Modal.Description>
            </Modal.Body>
            <Modal.Actions>
                <Modal.Action>OK</Modal.Action>
            </Modal.Actions>
        </Modal>
    )
}

export default NoTokenFoundModal
